use reqwest;
use serde::Deserialize;
use serde::Serialize;
use warp::{Filter, Rejection, Reply};

pub type Bonds = Vec<Bond>;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Bond {
    pub rate: String,
    pub fund_name: String,
    pub loan_period_max: String,
    pub repayment_freedom_max: String,
    pub isin_code: String,
}

#[tokio::main]
async fn main() {
    let message = "This application grabs the bonds rates from https://www.nordea.dk/privat/produkter/boliglaan/Kurser-realkreditlaan-kredit.html and displays them as a Prometheus metric endpoint at /metrics";

    let root = warp::path::end().map(move || message);
    let metrics = warp::path!("metrics").and_then(get_bonds);

    let routes = warp::get().and(root.or(metrics));

    println!("Started on port 8080");
    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}

async fn get_bonds() -> Result<impl Reply, Rejection> {
    let bonds = reqwest::get("https://ebolig.nordea.dk/wemapp/api/credit/fixedrate/bonds.json")
        .await
        .unwrap()
        .json::<Bonds>()
        .await
        .unwrap(); // TODO: Fix unwrap

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
