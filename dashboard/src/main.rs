use std::ops::Range;

use chrono::{NaiveDate, Utc};
use plotters::{coord::Shift, prelude::*};
use plotters_canvas::CanvasBackend;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use wasm_rs_dbg::dbg;
use yew::prelude::*;

use common::TestsuiteResult;

#[derive(Debug)]
enum Error {
    CacheAPI,
}

impl From<reqwasm::Error> for Error {
    fn from(_: reqwasm::Error) -> Self {
        Error::CacheAPI
    }
}

async fn fetch_testsuites(base_url: &str) -> Result<Vec<TestsuiteResult>, reqwasm::Error> {
    let url = format!("{}/api/testsuites", base_url);
    let response = reqwasm::http::Request::get(&url).send().await?;
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

#[derive(Clone, Copy)]
struct DateRange(NaiveDate, NaiveDate);

impl Iterator for DateRange {
    type Item = NaiveDate;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 <= self.1 {
            let next = self.0;
            self.0 = next.succ();

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

fn get_limits(testsuites: &[TestsuiteResult]) -> Range<u64> {
    // FIXME: Don't unwrap here
    0..testsuites
        .iter()
        .map(|run| run.results.tests)
        .max()
        .unwrap()
        + 1 // for extra comfortable viewing
}

fn graph(testsuites: &[TestsuiteResult]) {
    let root = get_root();
    root.fill(&WHITE).unwrap();

    let range = get_date_range(testsuites);
    let limits = get_limits(testsuites);

    let mut chart = ChartBuilder::on(&root)
        .caption("testsuites", ("sans-serif", 50).into_font())
        .margin(5u32)
        .x_label_area_size(30u32)
        .y_label_area_size(30u32)
        .build_cartesian_2d(0..testsuites.len(), limits)
        .unwrap();

    chart.configure_mesh().draw().unwrap();
    chart
        .draw_series(LineSeries::new(
            (range).enumerate().map(|(i, date)| {
                // There will always be a unique run per day
                // FIXME: Right?
                let to_show = testsuites.iter().find(|run| run.date == date).unwrap();

                (i, to_show.results.passes)
            }),
            &GREEN,
        ))
        .unwrap()
        .label("passes");

    chart
        .draw_series(LineSeries::new(
            (range).enumerate().map(|(i, date)| {
                // There will always be a unique run per day
                // FIXME: Right?
                let to_show = testsuites.iter().find(|run| run.date == date).unwrap();

                (i, to_show.results.failures)
            }),
            &RED,
        ))
        .unwrap()
        .label("failures");

    chart
        .draw_series(LineSeries::new(
            (range).enumerate().map(|(i, date)| {
                // There will always be a unique run per day
                // FIXME: Right?
                let to_show = testsuites.iter().find(|run| run.date == date).unwrap();

                (i, to_show.results.tests)
            }),
            &BLUE,
        ))
        .unwrap()
        .label("total");

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()
        .unwrap();
}

fn main() {
    // FIXME: Use environment variable instead?
    let url = "http://127.0.0.1:8000";
    dbg!(url);

    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    yew::start_app::<App>();

    spawn_local(async move {
        // FIXME: Can we unwrap here?
        let testsuites = fetch_testsuites(url).await.unwrap();

        graph(&testsuites);
    });
}
