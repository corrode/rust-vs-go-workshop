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

For the weather forecast, we will use the [OpenWeather API](https://openweathermap.org/api), because it is open source, easy to use, and offers a generous [free tier for non-commercial](https://open-meteo.com/en/pricing) use of up to 10,000 requests per day.

Let's get started!

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

There is a Go library for OpenWeather called [omgo]( https://github.com/HectorMalot/omgo), which we would use in a production service. However, we want to see what it takes to make an HTTP request in Go, so we will use the standard library instead.

Let's start with a simple function that makes an HTTP request to the OpenWeather API and returns the response body as a string:

```go
func getWeather(city string) (string, error) {
	url := fmt.Sprintf("https://api.openweathermap.org/data/2.5/weather?q=%s&appid=%s", city, apiKey)
	resp, err := http.Get(url)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()
	body, err := ioutil.ReadAll(resp.Body)
	if err != nil {
		return "", err
	}
	return string(body), nil
}
```

The function takes a city name as an argument and returns the response body as a string. Otherwise it returns an error in case the request fails. Simple enough.

For a start, let's call this function with a hard-coded city name and print the result to the console:

```go
func main() {
	weather, err := getWeather("London") // it will be rainy...
	if err != nil {
		log.Fatal(err)
	}
	fmt.Println(weather)
}
```


#### Routing

Routing is one of the most basic tasks of a web framework
and for our weather service, we need to support two routes:


#### Templates

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

