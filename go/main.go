package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"log"
	"net/http"
	"net/url"
)

type GeoResponse struct {
	Results []LatLong `json:"results"`
}

type LatLong struct {
	Latitude  float64 `json:"latitude"`
	Longitude float64 `json:"longitude"`
}

func getLatLong(city string) (*LatLong, error) {
	// Url encode the city name to make it safe for the request
	city = url.QueryEscape(city)
	url := fmt.Sprintf("https://geocoding-api.open-meteo.com/v1/search?name=%s&count=1&language=en&format=json", city)
	resp, err := http.Get(url)
	if err != nil {
		return nil, err
	}

	// Always close the response body to prevent resource leaks and ensure connection reuse.
	// Not closing it can lead to exhausting available file descriptors over time.
	defer resp.Body.Close()

	// Read the response body
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}

	// Parse the JSON response result
	var response GeoResponse
	err = json.Unmarshal(body, &response)
	if err != nil {
		return nil, err
	}

	// Check if we have at least one result
	if len(response.Results) < 1 {
		return nil, errors.New("no results found")
	}

	// Return the first result
	return &response.Results[0], nil
}

func getWeather(latLong LatLong) (string, error) {
	url := fmt.Sprintf("https://api.open-meteo.com/v1/forecast?latitude=%.6f&longitude=%.6f&hourly=temperature_2m", latLong.Latitude, latLong.Longitude)
	resp, err := http.Get(url)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return "", err
	}
	return string(body), nil
}

func main() {
	latlong, err := getLatLong("Berlin")
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("Latitude: %f, Longitude: %f\n", latlong.Latitude, latlong.Longitude)

	weather, err := getWeather(*latlong)
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("Weather: %s\n", weather)
}
