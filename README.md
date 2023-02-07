# bottleboard

The dashboard where we bottle all the testsuite results together :)

## Project architecture

### `bottlecache`

REST API exposing test-suite results through various endpoints and caching that
data.

#### endpoints

* `/api/testsuites`: List of testsuites for which results are stored
* `/api/testsuites/<key>`: List of testsuite results for that testsuite
* `/api/testsuites/<key>/<date>`: Testsuite result for that specific date

### `dashboard`

Front-end of the dashboard. Web Assembly app responsible for performing API calls
to the cache backend and displaying them properly.

### `common`

Set of common types to use in the front-end and back-end at the same time.

## Using/Deploying locally

You can run both the cache and front-end on the same machine locally. This makes
it really easy to test your changes.

First, deploy the cache:

```
# cd bottlecache
# cargo run -- --token <github access token>
```

The github token access is required.

The API is available on port 8000 of your local machine.

Then, deploy the frontend:

```
# cd dashboard
# trunk serve --open
```

This will open the application on your browser. By default, the API's URL is
your local machine.
