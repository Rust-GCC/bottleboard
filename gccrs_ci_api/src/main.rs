// FIXME: We need to figure out whether or not we want to update our "cache"
// directly from the API *or* rely on a cronjob.
// I think using the API to check the date of the most recent file in the cache
// folder, and then simply forking and running the `bottlecache` executable (or
// implementing its main function here directly) might be easier and cleaner.

#[rocket::get("/testsuites")]
async fn testsuites() -> &'static str {
    "testsuites"
}

#[rocket::get("/testsuites/<key>")]
async fn testsuites_by_key(key: &str) -> String {
    format!("testsuites by key: {}", key)
}

#[rocket::get("/")]
async fn index() -> &'static str {
    "hello!"
}

#[rocket::launch]
async fn rocket() -> _ {
    rocket::build().mount("/", rocket::routes![index, testsuites, testsuites_by_key])
}
