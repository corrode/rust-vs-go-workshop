# Go vs. Rust for web development

Go vs. Rust is a topic that keeps popping up and there has been a
lot written about it already. That is in part because developers are
looking for information to help them decide which language to use for
their next (web) project.

After all, both languages can be used to write fast and reliable
web services. On the other hand, their approaches to achieve that
are quite different and it is hard to find a good comparison that
tries to be fair to both languages.
This post is my attempt to give you an overview of the differences
between Go and Rust with a focus on web development. We will compare
the syntax, the web ecosystem, the way they handle typical web tasks
like routing, middleware, templating, and more.
We will also have a quick look at the concurrency models of both
languages and how they affect the way you write web applications.

By the end of this post, you should have a good idea of which
language is the right one for you.
Although that we are aware of our own biases and preferences, we
we will try to be as objective as possible and
highlight the strengths and weaknesses of *both* languages.

## Syntax

## Concurrency

## Web ecosystem

## Building a small web service

Many comparisons between Go and Rust focus on the syntax and
the language features. But in the end, what matters is how
easy it is to use them for a non-trivial project.

Since we are a platform as a service provider, we think that
we can contribute the most by showing you how to build a small
web service in both languages. We will use the same task and
popular libraries for both languages to compare the solutions
side-by-side.

We will cover the following topics:

- Routing
- Templating
- Database access
- Testing
- Deployment

We will leave out topics like client-side rendering
and focus on the server-side only.

### The task

Picking a task that is representative for web development is
not easy: On one hand, we want to keep it simple enough so
that we can focus on the language features and libraries.
On the other hand, we want to make sure that the task is
not *too* simple so that we can show how to use the language
features and libraries in a realistic setting.

We decided to build a *weather forecast service*. 
The user should be able to enter a city name and get the
current weather forecast for that city. The service should
also show a list of recently searched cities.

As we extend the service, we will add the following features:

- A simple UI to display the weather forecast
- A database to store recently searched cities

## The OpenWeather API

For the weather forecast, we will use the [Open-Meteo API](https://open-meteo.com/), because it is open source, easy to use, and offers a generous [free tier for non-commercial](https://open-meteo.com/en/pricing) use of up to 10,000 requests per day.

It has two endpoints that we will use:

- The [GeoCoding API](https://open-meteo.com/en/docs/geocoding-api) to get the coordinates of a city.
- The [Weather Forecast API](https://open-meteo.com/en/docs) to get the weather forecast for the given coordinates.

There are libraries for both Go ([omgo]( https://github.com/HectorMalot/omgo))
and Rust ([openmeteo](https://github.com/angelodlfrtr/open-meteo-rs))
, which we would use in a production service. However, for the sake of comparison, we want to see what it takes to make a "raw" HTTP request in
both languages and convert the response to an idiomatic data structure.

### A Go web service

#### Choosing a web framework

Being originally created to simplify building web services, Go has a
number of great web-related packages. 
If the standard library doesn't cover your needs, there are a number of
popular third-party web frameworks like [Gin](https://gin-gonic.com),
[Echo](https://echo.labstack.com/), or [Chi](https://go-chi.io/#/) to choose from.

Which one to pick is a matter of personal preference.
Some experienced Go developers prefer to use the standard
library and add a routing library like Chi on top of it.
Others prefer a more batteries-included approach and use
a full-featured framework like Gin or Echo.

Both options are fine, but for the purpose of this comparison,
we will choose [Gin](https://gin-gonic.com) because it is
one of the most popular frameworks and it supports all
the features we need for our weather service.

#### Making HTTP requests

Let's start with a simple function that makes an HTTP request to the OpenWeather API and returns the response body as a string:

```go
func getLatLong(city string) (*LatLong, error) {
	endpoint := fmt.Sprintf("https://geocoding-api.open-meteo.com/v1/search?name=%s&count=1&language=en&format=json", url.QueryEscape(city))
	resp, err := http.Get(endpoint)
	if err != nil {
		return nil, fmt.Errorf("error making request to Geo API: %w", err)
	}
	defer resp.Body.Close()

	var response GeoResponse
	if err := json.NewDecoder(resp.Body).Decode(&response); err != nil {
		return nil, fmt.Errorf("error decoding response: %w", err)
	}

	if len(response.Results) < 1 {
		return nil, errors.New("no results found")
	}

	return &response.Results[0], nil
}
```

The function takes a city name as an argument and returns the coordinates
of the city as a `LatLong` struct. 

Note how we handle errors after each step: We check if the HTTP request
was successful, if the response body could be decoded, and if the response
contains any results. If any of these steps fails, we return an error
and abort the function. So far, we just needed to use the standard library,
which is great.

The `defer` statement ensures that the response body is closed after the
function returns. This is a common pattern in Go to avoid resource leaks.
The compiler does not warn us in case we forget, so we need to be careful
here.

Error handling takes up a big part of the code. It is straightforward,
but it can be tedious to write and it can make the code harder to read.
On the plus side, the error handling is easy to follow and it is clear what happens in case of an error.

Since the API returns a JSON object with a list of results, we need to define a struct that matches that response:

```go
type GeoResponse struct {
	// A list of results; we only need the first one
	Results []LatLong `json:"results"`
}

type LatLong struct {
	Latitude  float64 `json:"latitude"`
	Longitude float64 `json:"longitude"`
}
```

The `json` tags tell the JSON decoder how to map the JSON fields to the struct fields. Extra fields in the JSON response are ignored by default.

Let's define another function that takes our `LatLong` struct and returns the weather forecast for that location:

```go
func getWeather(latLong LatLong) (string, error) {
	endpoint := fmt.Sprintf("https://api.open-meteo.com/v1/forecast?latitude=%.6f&longitude=%.6f&hourly=temperature_2m", latLong.Latitude, latLong.Longitude)
	resp, err := http.Get(endpoint)
	if err != nil {
		return "", fmt.Errorf("error making request to Weather API: %w", err)
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return "", fmt.Errorf("error reading response body: %w", err)
	}

	return string(body), nil
}
```

For a start, let's call these two functions in order and print the result:

```go
func main() func main() {
	latlong, err := getLatLong("London") // you know it will rain
	if err != nil {
		log.Fatalf("Failed to get latitude and longitude: %s", err)
	}
	fmt.Printf("Latitude: %f, Longitude: %f\n", latlong.Latitude, latlong.Longitude)

	weather, err := getWeather(*latlong)
	if err != nil {
		log.Fatalf("Failed to get weather: %s", err)
	}
	fmt.Printf("Weather: %s\n", weather)
}
```

This will print the following output:

```bash
Latitude: 51.508530, Longitude: -0.125740
Weather: {"latitude":51.5,"longitude":-0.120000124, ... }
```

Nice! We got the weather forecast for London.
Let's make this available as a web service.

#### Routing

Routing is one of the most basic tasks of a web framework.
First, let's add gin to our project.

```bash
go mod init github.com/user/goforecast
go get -u github.com/gin-gonic/gin
```

Then, let's replace our `main()` function with a server and a route that takes a city name as a parameter and returns the weather forecast for that city.

Gin supports path parameters and query parameters.

```go
// Path parameter
r.GET("/weather/:city", func(c *gin.Context) {
		city := c.Param("city")
		// ...
})

// Query parameter
r.GET("/weather", func(c *gin.Context) {
	city := c.Query("city")
	// ...
})
```go

Which one you want to use depends on your use case.
In our case, we want to submit the city name from a form in the end, so we will use a query parameter.

```go
func main() {
	r := gin.Default()

	r.GET("/weather", func(c *gin.Context) {
		city := c.Query("city")
		latlong, err := getLatLong(city)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
			return
		}

		weather, err := getWeather(*latlong)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
			return
		}

		c.JSON(http.StatusOK, gin.H{"weather": weather})
	})

	r.Run()
}
```

In a separate terminal, we can start the server with `go run .` and make a request to it:

```bash
curl "localhost:8080/weather?city=Hamburg"
```

And we get our weather forecast:

```json
{"weather":"{\"latitude\":53.550000,\"longitude\":10.000000, ... }
```

I like the log output and it's quite fast, too!

```bash
[GIN] 2023/09/09 - 19:27:20 | 200 |   190.75625ms |       127.0.0.1 | GET      "/weather?city=Hamburg"
[GIN] 2023/09/09 - 19:28:22 | 200 |   46.597791ms |       127.0.0.1 | GET      "/weather?city=Hamburg"
```

#### Templates

We got our endpoint, but raw JSON is not very useful to a normal user.
In a real-world application, we would probably serve the JSON response on an API endpoint (say `/api/v1/weather/:city`) and add a separate endpoint that returns the HTML page. For the sake of simplicity, we will just return the HTML page directly.

Let's add a simple HTML page that displays the weather forecast for a given city
as a table. We will use the `html/template` package from the standard library to render the HTML page.

First, let's add some structs for our view:

```go
type WeatherData struct 
type WeatherResponse struct {
	Latitude  float64 `json:"latitude"`
	Longitude float64 `json:"longitude"`
	Timezone  string  `json:"timezone"`
	Hourly    struct {
		Time          []string  `json:"time"`
		Temperature2m []float64 `json:"temperature_2m"`
	} `json:"hourly"`
}

type WeatherDisplay struct {
	City      string
	Forecasts []Forecast
}

type Forecast struct {
	Date        string
	Temperature string
}
```

This is just a direct mapping of the relevant fields in the JSON response to a struct. There are tools like [transform](https://transform.tools/json-to-go), which make conversion from JSON to Go structs easier. Take a look!

Next we define a function, which converts the raw JSON response from the weather API into our new `WeatherDisplay` struct:

```go
func extractWeatherData(city string, rawWeather string) (WeatherDisplay, error) {
	var weatherResponse WeatherResponse
	if err := json.Unmarshal([]byte(rawWeather), &weatherResponse); err != nil {
		return WeatherDisplay{}, fmt.Errorf("error decoding weather response: %w", err)
	}

	var forecasts []Forecast
	for i, t := range weatherResponse.Hourly.Time {
		date, err := time.Parse(time.RFC3339, t)
		if err != nil {
			return WeatherDisplay{}, err
		}
		forecast := Forecast{
			Date:        date.Format("Mon 15:04"),
			Temperature: fmt.Sprintf("%.1f°C", weatherResponse.Hourly.Temperature2m[i]),
		}
		forecasts = append(forecasts, forecast)
	}
	return WeatherDisplay{
		City:      city,
		Forecasts: forecasts,
	}, nil
}
```

Date handling is done with the built-in `time` package.
To learn more about date handling in Go, check out [this "Go by Example" article](https://blog.golang.org/using-go-modules).

We extend our route handler to render the HTML page:

```go
r.GET("/weather", func(c *gin.Context) {
	city := c.Query("city")
	latlong, err := getLatLong(city)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	weather, err := getWeather(*latlong)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	//////// NEW CODE STARTS HERE ////////
	weatherDisplay, err := extractWeatherData(city, weather)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}
	c.HTML(http.StatusOK, "weather.html", weatherDisplay)
	//////////////////////////////////////
})
```

Let's deal with the template next.
Create a template directory called `views` and tell Gin about it:

```go
r := gin.Default()
r.LoadHTMLGlob("views/*")
```

Finally, we can create a template file `weather.html` in the `views` directory:

```html
<!DOCTYPE html>
<html>
<head>
    <title>Weather Forecast</title>
</head>
<body>
    <h1>Weather for {{ .City }}</h1>
    <table border="1">
        <tr>
            <th>Date</th>
            <th>Temperature</th>
        </tr>
        {{ range .Forecasts }}
        <tr>
            <td>{{ .Date }}</td>
            <td>{{ .Temperature }}</td>
        </tr>
        {{ end }}
    </table>
</body>
</html>
```

(Take a look at the Gin documentation for more [details on how to use templates](https://gin-gonic.com/docs/examples/html-rendering/).)

With that, we have a working web service that returns the weather forecast for a given city as an HTML page!

Oh! Perhaps we also want to create an index page with an input field,
which allows us to enter a city name and displays the weather forecast for that city.

Let's add a new route handler for the index page:

```go
r.GET("/", func(c *gin.Context) {
	c.HTML(http.StatusOK, "index.html", nil)
})
```

And a new template file `index.html`:

```html
<!DOCTYPE html>
<html>
<head>
    <title>Weather Forecast</title>
</head>
<body>
    <h1>Weather Forecast</h1>
    <form action="/weather" method="get">
        <label for="city">City:</label>
        <input type="text" id="city" name="city">
        <input type="submit" value="Submit">
    </form>
</body>
</html>
```

Now we can start our web service and open [http://localhost:8080](http://localhost:8080) in our browser:

![index page](./images/index.png)

As an exercise, you can add some styling to the HTML page,
but since we care more about the backend, we will leave it at that.

#### Database access

Our service fetches the latitude and longitude for a given city from an external API on every single request. That's probably fine in the
beginning, but eventually we might want to cache the results in a database
to avoid unnecessary API calls.

To do so, let's add a database to our web service.
We will use [PostgreSQL](https://www.postgresql.org/) as our database and [pgx](https://github.com/jackc/pgx) as the database driver.

First, we create a file named `init.sql`, which will be used to initialize our database:

```sql
CREATE TABLE IF NOT EXISTS cities (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    lat NUMERIC NOT NULL,
    long NUMERIC NOT NULL
);

CREATE INDEX IF NOT EXISTS cities_name_idx ON cities (name);
```

We store the latitude and longitude for a given city.
The `SERIAL` type is a PostgreSQL auto-incrementing integer. Otherwise we would
have to generate the IDs ourselves on insert.
To make things fast, we will also add an index on the `name` column.

It's probably easiest to use Docker or any of the cloud providers.
At the end of the day, you just need *a database URL*, which you can pass to your web service as an environment variable.

We won't go into the details of setting up a database here, but a simple way to get a PostgreSQL database running with Docker locally is:

```
docker run -p 5432:5432 -e POSTGRES_USER=forecast -e POSTGRES_PASSWORD=forecast -e POSTGRES_DB=forecast -v `pwd`/init.sql:/docker-entrypoint-initdb.d/index.sql -d postgres
export DATABASE_URL="postgres://forecast:forecast@localhost:5432/forecast?sslmode=disable"
```

However once we have our database, we need to add the [sqlx](https://github.com/jmoiron/sqlx) dependency to our `go.mod` file:

```go
go get github.com/jmoiron/sqlx
```

We can now use the `sqlx` package to connect to our database by using the connection string from the `DATABASE_URL` environment variable:

```go
_ = sqlx.MustConnect("postgres", os.Getenv("DATABASE_URL"))
```

And with that, we have a database connection!

Let's add a function to insert a city into our database.
We will use our `LatLong` struct from earlier.

```go
func insertCity(db *sqlx.DB, name string, latLong LatLong) error {
	_, err := db.Exec("INSERT INTO cities (name, lat, long) VALUES ($1, $2, $3)", name, latLong.Latitude, latLong.Longitude)
	return err
}
```

Let's rename our old `getLatLong` function to `fetchLatLong` and add a new `getLatLong` function, which uses the database instead of the external API:

```go
func getLatLong(db *sqlx.DB, name string) (*LatLong, error) {
	var latLong *LatLong
	err := db.Get(&latLong, "SELECT lat, long FROM cities WHERE name = $1", name)
	if err == nil {
		return latLong, nil
	}

	latLong, err = fetchLatLong(name)
	if err != nil {
		return nil, err
	}

	err = insertCity(db, name, *latLong)
	if err != nil {
		return nil, err
	}

	return latLong, nil
}
```

Here we directly pass the `db` connection to our `getLatLong` function.
In a real application, we should decouple the database access from the API logic, to make testing possible.
We would probably also use an in-memory-cache to avoid unnecessary database calls. This is just to compare database access in Go and Rust.

We need to update our handler:

```go
r.GET("/weather", func(c *gin.Context) {
	city := c.Query("city")
	// Pass in the db
	latlong, err := getLatLong(db, city)
	// ...
})
```

With that, we have a working web service that stores the latitude and longitude for a given city in a database and fetches it from there on subsequent requests.

#### Middleware

The last bit is to add some middleware to our web service.
We already got some nice logging for free from Gin.

Let's add a basic-auth middleware and protect our `/stats` endpoint,
which we will use to print the last search queries.

```go
r.GET("/stats", gin.BasicAuth(gin.Accounts{
		"forecast": "forecast",
	}), func(c *gin.Context) {
		// rest of the handler
	}
)
```

That's it!

Pro tip: you can also [group routes together](https://jonathanmh.com/go-gin-http-basic-auth/) to apply authentication to multiple routes at once.

Here's the logic to fetch the last search queries from the database:

```go
func getLastCities(db *sqlx.DB) ([]string, error) {
	var cities []string
	err := db.Select(&cities, "SELECT name FROM cities ORDER BY id DESC LIMIT 10")
	if err != nil {
		return nil, err
	}
	return cities, nil
}
```

Now let's wire up our `/stats` endpoint to print the last search queries:

```go
r.GET("/stats", gin.BasicAuth(gin.Accounts{
		"forecast": "forecast",
	}), func(c *gin.Context) {
		cities, err := getLastCities(db)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
			return
		}
		c.HTML(http.StatusOK, "stats.html", cities)
})
```

Our `stats.html` template is simple enough:

```html
<!DOCTYPE html>
<html>

<head>
    <title>Latest Queries</title>
</head>

<body>
    <h1>Latest Lat/Long Lookups</h1>
    <table border="1">
        <tr>
            <th>Cities</th>
        </tr>
        {{ range . }}
        <tr>
            <td>{{ . }}</td>
        </tr>
        {{ end }}
    </table>
</body>

</html>
```

And with that, we have a working web service! Congratulations!

We have achieved the following:
- A web service that fetches the latitude and longitude for a given city from an external API
- Stores the latitude and longitude in a database
- Fetches the latitude and longitude from the database on subsequent requests
- Prints the last search queries on the `/stats` endpoint
- Basic-auth to protect the `/stats` endpoint
- Uses middleware to log requests
- Templates to render HTML

That's quite a lot of functionality for a few lines of code!
Let's see how Rust stacks up!

### A Rust web service

Historically, Rust didn't have a good story for web services.
There were a few frameworks, but they were quite low-level.
Only recently, with the emergence of async/await, did the Rust web ecosystem really take off. Suddenly, it was possible to write highly performant web services without a garbage collector and with fearless concurrency.

We will see how Rust compares to Go in terms of ergonomics, performance and safety. But first, we need to choose a web framework.

#### Axum or Actix?

Actix is a very popular web framework in the Rust community.
It is based on the actor model and uses async/await under the hood.
In benchmark, [it regularly shows up as one of the fastest web frameworks in the world](https://www.techempower.com/benchmarks/#section=data-r21&test=composite).

[Axum](https://github.com/tokio-rs/axum) is a new web framework that is based on [tower](https://github.com/tower-rs/tower), a library for building async services. It has received a lot attention in the Rust community and is quickly gaining popularity. It is also based on async/await.

Both frameworks are very similar in terms of ergonomics and performance.
They both support middleware and routing. Each of them would be a good choice for our web service, but we will go with Axum, because it ties in nicely with the rest of the ecosystem and has gotten a lot of attention recently.

#### Routing

Let's start the project with a `cargo new forecast` and add the following dependencies to our `Cargo.toml`:

```toml
[dependencies]
# web framework
axum = "0.6.20"
# async HTTP client
reqwest = { version = "0.11.20", features = ["json"] }
# serialization/deserialization  for JSON
serde = "1.0.188"
# database access
sqlx = "0.7.1"
# async runtime
tokio = { version = "1.32.0", features = ["full"] }
```

Let's create a little skeleton for our web service, which doesn't do much.

```rust
use std::net::SocketAddr;

use axum::{routing::get, Router};

// basic handler that responds with a static string
async fn index() -> &'static str {
    "Index"
}

async fn weather() -> &'static str {
    "Weather"
}

async fn stats() -> &'static str {
    "Stats"
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/weather", get(weather))
        .route("/stats", get(stats));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

The `main` function is pretty straightforward. We create a router and bind it to a socket address. The `index`, `weather` and `stats` functions are our handlers. They are async functions that return a string. We will replace them with actual logic later.

Let's run the web service with `cargo run` and see what happens.

```bash
$ curl localhost:3000
Index
$ curl localhost:3000/weather
Weather
$ curl localhost:3000/stats
Stats
```

Okay, that works. Let's add some actual logic to our handlers.

Let's write a function that fetches the latitude and longitude for a given city from an external API.

Here are the structs representing the response from the API:

```rust
use serde::Deserialize;

pub struct GeoResponse {
    pub results: Vec<LatLong>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LatLong {
    pub latitude: f64,
    pub longitude: f64,
}
```

In comparison to Go, we don't use tags to specify the field names. Instead, we use the `#[derive(Deserialize)]` attribute to automatically derive the `Deserialize` trait for our structs. These derive macros are very powerful and allow us to do a lot of things with very little code. It is a very common pattern in Rust.

Let's use the new types to fetch the latitude and longitude for a given city:

```rust
async fn fetch_lat_long(city: &str) -> Result<LatLong, Box<dyn std::error::Error>> {
    let endpoint = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
        city
    );
    let response = reqwest::get(&endpoint).await?.json::<GeoResponse>().await?;
    response
        .results
        .get(0)
        .cloned()
        .ok_or("No results found".into())
}
```

The code is a bit less verbose than the Go version. We don't have to
write `if err != nil` constructs, because we can use the `?` operator to
propagate errors. This is also mandatory, as each step returns a
`Result` type. If we don't handle the error, we won't get access to the
value.

That last part might look a bit unfamiliar:

```rust
response
    .results
    .get(0)
    .cloned()
    .ok_or("No results found".into())
```

A few things are happening here:

- `response.results.get(0)` returns an `Option<&LatLong>`. It is an
  `Option` because the `get` function might return `None` if the vector
  is empty.
- `cloned()` clones the value inside the `Option` and converts the
  `Option<&LatLong>` into an `Option<LatLong>`. This is necessary,
  because we want to return a `LatLong` and not a reference. Otherwise,
  we would have to add a lifetime specifier to the function signature and
  it makes the code less readable.
- `ok_or("No results found".into())` converts the `Option<LatLong>` into
  a `Result<LatLong, Box<dyn std::error::Error>>`. If the `Option` is
  `None`, it will return the error message. The `into()` function
  converts the string into a `Box<dyn std::error::Error>`.

An alternative way to write this would be:

```rust
match response.results.get(0) {
    Some(lat_long) => Ok(lat_long.clone()),
    None => Err("No results found".into()),
}
```

It is a matter of taste which version you prefer. 
  
Rust is an expression-based language, which means that we don't have to
use `return` to return a value from a function. Instead, the last value
of a function is returned.

We can now update our `weather` function to use `fetch_lat_long`.

Our first attempt might look like this:

```rust
async fn weather(city: String) -> String {
    println!("city: {}", city);
    let lat_long = fetch_lat_long(&city).await.unwrap();
    format!("{}: {}, {}", city, lat_long.latitude, lat_long.longitude)
}
```

First we print the city to the console, then we fetch the latitude and
longitude and unwrap (i.e. "unpack") the result. If the result is an error, the program will panic. This is not ideal, but we will fix it later.

We then use the latitude and longitude to create a string and return it.

Let's run the program and see what happens:

```bash
curl -v "localhost:3000/weather?city=Berlin"
*   Trying 127.0.0.1:3000...
* Connected to localhost (127.0.0.1) port 3000 (#0)
> GET /weather?city=Berlin HTTP/1.1
> Host: localhost:3000
> User-Agent: curl/8.1.2
> Accept: */*
> 
* Empty reply from server
* Closing connection 0
curl: (52) Empty reply from server
```

Furthermore, we get this output:

```bash
city:
```

The `city` parameter is empty. What happened?

The problem is that we are using the `String` type for the `city`
parameter. This type is not a valid [extractor](https://docs.rs/axum/latest/axum/extract/index.html).

We can use the `Query` extractor instead:

```rust
async fn weather(Query(params): Query<HashMap<String, String>>) -> String {
    let city = params.get("city").unwrap();
    let lat_long = fetch_lat_long(&city).await.unwrap();
    format!("{}: {}, {}", *city, lat_long.latitude, lat_long.longitude)
}
```

This will work, but it is not very idiomatic. We have to `unwrap`
the `Option` to get the city. We also need to pass `*city` to the
`format!` macro to get the value instead of the reference. (It's
called "dereferencing" in Rust lingo.)

We could create a struct that represents the query parameters:

```rust
#[derive(Deserialize)]
pub struct WeatherQuery {
    pub city: String,
}
```

We can then use this struct as an extractor and avoid the `unwrap`:

```rust
async fn weather(Query(params): Query<WeatherQuery>) -> String {
    let lat_long = fetch_lat_long(&params.city).await.unwrap();
    format!("{}: {}, {}", params.city, lat_long.latitude, lat_long.longitude)
}
```

Cleaner!
It's a little more involved than the Go version, but it's also more
type-safe. You can imagine that we can add constraints to the struct
to add validation. For example, we could require that the city is at
least 3 characters long.

Now about the `unwrap` in the `weather` function.
Ideally, we would return an error if the city is not found. We can do
this by changing our return type.

In axum, anything that implements [`IntoResponse`](https://docs.rs/axum/latest/axum/response/trait.IntoResponse.html) can be returned from handlers, however it is advisable to return
a concrete type, as there are 
[some caveats with returning `impl IntoResponse`] (https://docs.rs/axum/latest/axum/response/index.html)

In our case, we can return a `Result` type:

```rust
async fn weather(Query(params): Query<WeatherQuery>) -> Result<String, StatusCode> {
    let lat_long = fetch_lat_long(&params.city)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(format!(
        "{}: {}, {}",
        params.city, lat_long.latitude, lat_long.longitude
    ))
}
```

This will return a `404` status code if the city is not found.
We use `map_err` to convert the error into a `StatusCode` and then
use the `?` operator to propagate the error.

It would be equally fine and maybe a little easier to read to use `match` instead of `map_err`:

```rust
async fn weather(Query(params): Query<WeatherQuery>) -> Result<String, StatusCode> {
    match fetch_lat_long(&params.city).await {
        Ok(lat_long) => Ok(format!(
            "{}: {}, {}",
            params.city, lat_long.latitude, lat_long.longitude
        )),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}
```

In Rust, there are usually multiple ways to do things. It's a matter of
taste which version you prefer.

In any case, let's test our program:

```bash
curl "localhost:3000/weather?city=Berlin"
Berlin: 52.52437, 13.41053
```

and

```bash
curl -I "localhost:3000/weather?city=abcdedfg"
HTTP/1.1 404 Not Found
```

Let's write second function, which will return the weather for a given
latitude and longitude:

```rust

#### Templates 

#### Database access

#### Middleware

## Deployment

## Which language is right for you?

- Go: 
  * easy to learn, fast, good for web services
  * batteries included. We did a lot with just the standard library.
  * Our only dependency was Gin, which is a very popular web framework.

## Further reading

