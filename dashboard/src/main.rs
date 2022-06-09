use chrono::NaiveDate;
use plotters::{coord::Shift, prelude::*};
use plotters_canvas::CanvasBackend;
use structopt::StructOpt;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use common::TestsuiteResult;

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(short, long, help = "URL to make API calls to")]
    api_url: String,
}

#[derive(Debug)]
enum Error {
    CacheAPI,
}

impl From<reqwest::Error> for Error {
    fn from(_: reqwest::Error) -> Self {
        Error::CacheAPI
    }
}

async fn fetch_testsuites(base_url: &str) -> Result<Vec<TestsuiteResult>, reqwest::Error> {
    let url = format!("{}/api/testsuites", base_url);
    let response = reqwest::get(url).await?;
    let testsuites: Vec<TestsuiteResult> = response.json().await?;

    Ok(testsuites)
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <div>
            <h3>{ "Hello there!" }</h3>
            <canvas id="canvas"
            width="500" height="300"></canvas>
        </div>
    }
}

// FIXME: Ugly ass return type
fn get_root() -> DrawingArea<CanvasBackend, Shift> {
    // FIXME: Don't unwrap
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    CanvasBackend::with_canvas_object(canvas)
        .unwrap()
        .into_drawing_area()
}

struct DateRange(NaiveDate, NaiveDate);

impl Iterator for DateRange {
    type Item = NaiveDate;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 <= self.1 {
            let next = self.0.succ();
            self.0 = next;

            Some(next)
        } else {
            None
        }
    }
}

fn get_date_range(testsuites: &[TestsuiteResult]) -> DateRange {
    // FIXME: This can panic if the array is empty
    let mut lo: NaiveDate = testsuites[0].date;
    let mut hi: NaiveDate = testsuites[0].date;

    testsuites.iter().for_each(|run| {
        if run.date < lo {
            lo = run.date
        };
        if run.date > hi {
            hi = run.date
        }
    });

    DateRange(lo, hi)
}

fn graph(testsuites: &[TestsuiteResult]) {
    let root = get_root();
    root.fill(&WHITE).unwrap();

    let range = get_date_range(testsuites);

    let mut chart = ChartBuilder::on(&root)
        .caption("testsuites", ("sans-serif", 50).into_font())
        .margin(5u32)
        .x_label_area_size(30u32)
        .y_label_area_size(30u32)
        .build_cartesian_2d(0u64..5u64, 0u64..5u64)
        .unwrap();

    chart.configure_mesh().draw().unwrap();

    chart
        .draw_series(LineSeries::new(
            (range).map(|date| {
                // There will always be a unique run per day
                let to_show = testsuites.iter().find(|run| run.date == date).unwrap();

                (to_show.results.passes, to_show.results.passes)
            }),
            &RED,
        ))
        .unwrap()
        .label("y = x^2")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()
        .unwrap();
}

fn main() {
    let args = Args::from_args();

    yew::start_app::<App>();

    spawn_local(async move {
        // FIXME: Can we unwrap here?
        let testsuites = fetch_testsuites(&args.api_url).await.unwrap();

        graph(&testsuites);
    });
}
