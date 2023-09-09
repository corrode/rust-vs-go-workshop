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

#### Middleware



#### Database access

#### Testing

#### Deployment

### A Rust web service

#### Axum or Actix?

#### Routing

#### Templates 

#### Middleware

#### Database access

#### Testing

#### Deployment

## Which language is right for you?

## Further reading

