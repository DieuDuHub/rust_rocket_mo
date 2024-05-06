use reqwest;
use std::env;
extern crate config;

/// Initialize configuration
/// PROXY_URL  variable has to be configure to reach Configuration server
/// Toml file will be read from there
pub async fn init_configuration(filepath: String) -> Result<config::Config, String> {
    let o = match env::var("PROXY_URL") {
        Ok(o) => o,
        Err(e) => {
            return Err(format!("PROXY_URL path missing: {}. Should be like export PROXY_URL=localhost:8081/config/getconfigfile/mo-config/back-mo/infra.toml or http://localhost:8888/rustmo/default/main/rustmo.properties if using JAva Config Server", e));
        }
    };

    let url = format!("http://{}/{}", o, filepath);

    // Prepare loading of conf API from conf Server
    let data = reqwest::get(url.as_str()).await;

    //	println!("rec : {}", data.unwrap().text().unwrap());
    let exploitfile = data.unwrap().text().await.unwrap();

    let builder = config::Config::builder().add_source(config::File::from_str(
        exploitfile.as_str(),
        config::FileFormat::Toml,
    ));

    let settings = match builder.build() {
        Ok(config) => config,
        Err(e) => {
            return Err(format!(
                "Error parsing configuration file {}",
                e.to_string()
            ))
        }
    };
    Ok(settings)
}

#[async_test]
async fn test_init_configuration() {
    let settings = init_configuration(String::from("")).await.unwrap();
    //settings.get::<u16>("db_port").unwrap()
    //settings.get::<String>("db_database").unwrap()

    assert_eq!(settings.get::<String>("db_usr").unwrap(), "mdeb");
}
