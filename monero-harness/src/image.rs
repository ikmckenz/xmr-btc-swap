use testcontainers::{core::WaitFor, Image, ImageArgs};

pub const MONEROD_DAEMON_CONTAINER_NAME: &str = "monerod";
pub const MONEROD_DEFAULT_NETWORK: &str = "monero_network";

const NAME: &str = "rinocommunity/monero";
const TAG: &str = "v0.18.1.2";


/// The port we use for all RPC communication.
///
/// This is the default when running monerod.
/// For `monero-wallet-rpc` we always need to specify a port. To make things
/// simpler, we just specify the same one. They are in different containers so
/// this doesn't matter.
pub const RPC_PORT: u16 = 18081;

#[derive(Debug, Clone, Copy, Default)]
pub struct Monerod {}

impl ImageArgs for Monerod {
    fn into_iterator(self) -> Box<dyn Iterator<Item=String>> {
        Box::new(MonerodArgs::default().into_iter())
    }
}

impl Image for Monerod {
    type Args = Self;

    fn name(&self) -> String {
        NAME.to_owned()
    }

    fn tag(&self) -> String {
        TAG.to_owned()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![
            WaitFor::message_on_stdout("RPC server started ok"),
        ]
    }

    fn entrypoint(&self) -> Option<String> {
        Some("".to_owned()) // an empty entrypoint disables the entrypoint
                            // script and gives us full control
    }
}

#[derive(Debug, Clone, Default)]
pub struct MoneroWalletRpc {
    wallet_name: String,
    daemon_address: String,
}

impl ImageArgs for MoneroWalletRpc {
    fn into_iterator(self) -> Box<dyn Iterator<Item=String>> {
        tracing::info!("Into iterator {}", self.wallet_name.clone());
        Box::new(MoneroWalletRpcArgs::new(self.wallet_name.clone(), self.daemon_address.clone()).into_iter())
    }
}

impl Image for MoneroWalletRpc {
    type Args = Self;

    fn name(&self) -> String {
        NAME.to_owned()
    }

    fn tag(&self) -> String {
        TAG.to_owned()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![
            WaitFor::message_on_stdout("Run server thread name: RPC"),
        ]
    }

    fn entrypoint(&self) -> Option<String> {
        Some("".to_owned()) // an empty entrypoint disables the entrypoint
                            // script and gives us full control
    }
}

impl MoneroWalletRpc {
    pub fn new(wallet_name: String, daemon_address: String) -> Self {
        tracing::info!("MoneroWalletRpc::new {}", wallet_name);
        Self {
            wallet_name,
            daemon_address,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MonerodArgs {
    pub regtest: bool,
    pub offline: bool,
    pub rpc_payment_allow_free_loopback: bool,
    pub confirm_external_bind: bool,
    pub no_igd: bool,
    pub hide_my_port: bool,
    pub rpc_bind_ip: String,
    pub fixed_difficulty: u32,
    pub data_dir: String,
}

impl Default for MonerodArgs {
    fn default() -> Self {
        Self {
            regtest: true,
            offline: true,
            rpc_payment_allow_free_loopback: true,
            confirm_external_bind: true,
            no_igd: true,
            hide_my_port: true,
            rpc_bind_ip: "0.0.0.0".to_string(),
            fixed_difficulty: 1,
            data_dir: "/monero".to_string(),
        }
    }
}

impl IntoIterator for MonerodArgs {
    type Item = String;
    type IntoIter = ::std::vec::IntoIter<String>;

    fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        let mut args = vec![
            "monerod".to_string(),
            "--log-level=4".to_string(),
            "--non-interactive".to_string(),
        ];

        if self.regtest {
            args.push("--regtest".to_string())
        }

        if self.offline {
            args.push("--offline".to_string())
        }

        if self.rpc_payment_allow_free_loopback {
            args.push("--rpc-payment-allow-free-loopback".to_string())
        }

        if self.confirm_external_bind {
            args.push("--confirm-external-bind".to_string())
        }

        if self.no_igd {
            args.push("--no-igd".to_string())
        }

        if self.hide_my_port {
            args.push("--hide-my-port".to_string())
        }

        if !self.rpc_bind_ip.is_empty() {
            args.push(format!("--rpc-bind-ip={}", self.rpc_bind_ip));
        }

        if !self.data_dir.is_empty() {
            args.push(format!("--data-dir={}", self.data_dir));
        }

        if self.fixed_difficulty != 0 {
            args.push(format!("--fixed-difficulty={}", self.fixed_difficulty));
        }

        args.into_iter()
    }
}

#[derive(Debug, Clone)]
pub struct MoneroWalletRpcArgs {
    pub wallet_name: String,
    pub disable_rpc_login: bool,
    pub confirm_external_bind: bool,
    pub wallet_dir: String,
    pub rpc_bind_ip: String,
    pub daemon_address: String,
}

impl Default for MoneroWalletRpcArgs {
    fn default() -> Self {
        unimplemented!("A default instance for `MoneroWalletRpc` doesn't make sense because we always need to connect to a node.")
    }
}

impl MoneroWalletRpcArgs {
    pub fn new(wallet_name: String, daemon_address: String) -> Self {
        Self {
            wallet_name: wallet_name.clone(),
            disable_rpc_login: true,
            confirm_external_bind: true,
            wallet_dir: wallet_name,
            rpc_bind_ip: "0.0.0.0".into(),
            daemon_address,
        }
    }
}

impl IntoIterator for MoneroWalletRpcArgs {
    type Item = String;
    type IntoIter = ::std::vec::IntoIter<String>;

    fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        let mut args = vec![
            "monero-wallet-rpc".to_string(),
            format!("--wallet-dir={}", self.wallet_dir),
            format!("--daemon-address={}", self.daemon_address),
            format!("--rpc-bind-port={}", RPC_PORT),
            "--log-level=4".to_string(),
            "--allow-mismatched-daemon-version".to_string(), /* https://github.com/monero-project/monero/issues/8600 */
        ];

        if self.disable_rpc_login {
            args.push("--disable-rpc-login".to_string())
        }

        if self.confirm_external_bind {
            args.push("--confirm-external-bind".to_string())
        }

        if !self.rpc_bind_ip.is_empty() {
            args.push(format!("--rpc-bind-ip={}", self.rpc_bind_ip));
        }

        args.into_iter()
    }
}
