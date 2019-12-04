use {phonenumber::PhoneNumber, qiwi::*, serde::*, std::path::*, structopt::*, tokio::stream::*};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Config {
    phone: String,
    token: String,
}

fn config_location() -> PathBuf {
    let mut path = xdg::BaseDirectories::new().unwrap().get_config_home();
    path.push("qiwi-cli/config.toml");

    path
}

#[derive(Debug, StructOpt)]
enum UnauthorizedCmd {
    /// Authorize client
    Login,
}

#[derive(Debug, StructOpt)]
#[allow(clippy::large_enum_variant)]
enum AuthorizedCmd {
    /// Reauthorize client
    Login,
    /// Get profile info,
    ProfileInfo,
    /// Get payment history,
    PaymentHistory,
    CommissionInfo {
        provider: ProviderId,
    },
}

async fn do_authorize() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut stdin = tokio_util::codec::FramedRead::new(
        tokio::io::stdin(),
        tokio_util::codec::LinesCodec::new(),
    );

    println!("Please enter user ID");

    let phone = stdin
        .next()
        .await
        .unwrap_or_else(|| std::process::exit(0))?
        .parse::<PhoneNumber>()?
        .to_string();

    println!("Please enter your token");

    let token = stdin
        .next()
        .await
        .unwrap_or_else(|| std::process::exit(0))?;

    let path = config_location();
    println!("Saving token on disk to {}", path.to_string_lossy());
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    tokio::fs::write(path, toml::to_vec(&Config { phone, token })?).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();

    let config = async move {
        if let Ok(data) = tokio::fs::read(config_location()).await {
            if let Ok(config) = toml::from_slice::<Config>(&data) {
                return Some(config);
            }
        }

        return None;
    }
    .await;

    match config {
        None => match UnauthorizedCmd::from_args() {
            UnauthorizedCmd::Login => do_authorize().await?,
        },
        Some(config) => match AuthorizedCmd::from_args() {
            AuthorizedCmd::Login => do_authorize().await?,
            other => {
                println!("Using config {:?}", config);
                let client = Client::new(config.phone.parse()?, config.token);
                let _ = client;
                match other {
                    AuthorizedCmd::ProfileInfo => {
                        let profile_info = client.profile_info().await?;
                        println!("Profile info:");
                        println!("{:?}", profile_info);
                    }
                    AuthorizedCmd::PaymentHistory => {
                        while let Some(entry) = client.payment_history().next().await.transpose()? {
                            println!("{:?}", entry);
                        }
                    }
                    AuthorizedCmd::CommissionInfo { provider } => {
                        println!("{:?}", client.commission_info(provider).await?)
                    }
                    other => unimplemented!("{:?}", other),
                }
            }
        },
    };

    Ok(())
}
