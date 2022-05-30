#[macro_use]
extern crate rocket;

#[macro_use]
extern crate log;

use pretty_env_logger;
use reqwest;
use rocket::{http, request, response};
use serde::{Deserialize, Serialize};
use thiserror::Error;

type Bonds = Vec<Bond>;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Bond {
    rate: String,
    fund_name: String,
    loan_period_max: String,
    repayment_freedom_max: String,
    isin_code: String,
}

#[derive(Error, Debug)]
enum CustomError {
    #[error("Reqwest Error")]
    Reqwest(#[from] reqwest::Error),
}

impl<'r, 'o: 'r> response::Responder<'r, 'o> for CustomError {
    fn respond_to(self, req: &request::Request) -> response::Result<'o> {
        match self {
            Self::Reqwest(error) => {
                error!("{}", error);
                http::Status::InternalServerError.respond_to(req)
            }
        }
    }
}

#[get("/")]
fn index() -> &'static str {
    "nordea-kredit-rate metrics"
}

#[get("/metrics")]
async fn metrics() -> Result<String, CustomError> {
    let bonds = reqwest::get("https://ebolig.nordea.dk/wemapp/api/credit/fixedrate/bonds.json")
        .await?
        .error_for_status()?
        .json::<Bonds>()
        .await?;

    let mut html: Vec<String> = Vec::new();

    html.push("# HELP nordea_kredit gauge Rates on Nordea Kredit bonds".to_owned());
    html.push("# TYPE nordea_kredit gauge".to_owned());

    for bond in bonds.iter() {
        let fund_name = format!("fund_name=\"{}\"", bond.fund_name);
        let isin_code = format!("isin_code=\"{}\"", bond.isin_code);
        let loan_period_max = format!("loan_period_max=\"{}\"", bond.loan_period_max);
        let rate = bond.rate.replace("*&nbsp;", "").replace(",", ".");
        let repayment_freedom_max = match bond.repayment_freedom_max.parse::<u8>() {
            Ok(x) => format!("repayment_freedom_max=\"{}\"", x),
            Err(_) => format!("repayment_freedom_max=\"0\""),
        };

        let metric = format!(
            "nordea_kredit{{{},{},{},{}}}{}",
            isin_code, loan_period_max, repayment_freedom_max, fund_name, rate
        );
        html.push(metric);
    }
    let result = html.join("\n");
    Ok(result)
}

#[launch]
fn rocket() -> _ {
    pretty_env_logger::init_timed();
    rocket::build().mount("/", routes![index, metrics])
}
