# umfeld

A small HTTP service for **ambient smart-home displays** that returns **local time, weather
and fuel prices for a named place**.

Everything comes back as a single JSON document: one request, one response, nothing to stitch
together on the client. It is designed to power glanceable displays such as wall panels,
e-ink dashboards, and kiosk monitors: the display stays simple and only has to render the
data it receives.

`Umfeld` is German for *surroundings*, used here in the geographic sense: the area around a
point on the map.

## Scope

Place lookup and weather work **worldwide**. Fuel prices are **Germany-only**, because they
come from the MTS-K via Tankerkönig, which covers German stations exclusively.

The service is written to be adapted rather than used as-is: see
[Adapting to another region](#adapting-to-another-region). Everything is in English, but the
defaults assume a German deployment — the geocoder is queried with `language=de`, so place
names and countries come back in German.

## Quick start

Requires a Rust toolchain supporting edition 2024 (1.85 or newer) and a free
[Tankerkönig API key](https://creativecommons.tankerkoenig.de/).

```sh
git clone https://github.com/CedricFauth/umfeld.git && cd umfeld
echo 'TANKER_API_KEY=your-key-here' > .env
cargo run
```

The service listens on `0.0.0.0:3000`; set `PORT` to use a different port. The key is read from the environment, with `.env`
loaded as a convenience for local development; `.env` is gitignored, so your key will not
be committed.

```sh
curl 'http://localhost:3000/snapshot?place=Dresden'
```

`place` must be URL-encoded. To pick the right one among places with the same name, add
the country after a comma, e.g. `place=Bergisch+Gladbach,Deutschland` or
`place=Paris,Frankreich`. Without a country the geocoder returns its best match, which is
usually the largest place with that name.

`GET /` returns a short HTML page describing the service. It is not rate-limited.

## Example

Everything arrives in one envelope, abridged here — three days of forecast, all stations
within 7 km sorted by distance:

```jsonc
{
  "success": true,
  "error": null,
  "datetime": "2026-09-27T18:04:22.176215+02:00",   // in the place's local timezone
  "attribution": [ /* CC BY notices, see below */ ],
  "place": { "name": "Dresden", "latitude": 51.05089, "longitude": 13.73832,
             "elevation": 116.0, "timezone": "Europe/Berlin", "country": "Deutschland",
             "country_code": "DE", "population": 556227 },
  "weather": {
    "current": { "time": "2026-09-27T18:00", "temperature_2m": 21.2,
                 "apparent_temperature": 19.0, "precipitation": 0.0 },
    "hourly":  { "time": […], "temperature_2m": […], "apparent_temperature": […],
                 "precipitation_probability": […], "precipitation": […] },
    "daily":   { "time": […], "temperature_2m_max": […], "temperature_2m_min": […],
                 "precipitation_probability_max": […], "precipitation_hours": […],
                 "precipitation_sum": […], "uv_index_max": […], "sunrise": […], "sunset": […] }
  },
  "fuel": [
    { "name": "Shell Dresden Loebtauer Str. 28/30", "brand": "Shell",
      "street": "Loebtauer Str.", "house_number": "28/30", "post_code": "01159",
      "place": "Dresden", "latitude": 51.051987, "longitude": 13.714982,
      "distance": 1.6, "diesel": 2.429, "e5": 2.349, "e10": 2.289, "is_open": true }
  ]
}
```

Metric units throughout — °C, mm, %, km, EUR per litre — and ISO 8601 timestamps in the
place's local time. Failures keep the same shape with `success: false` and a message in
`error`, so check that field rather than the status code. Requests are limited to one per
60 seconds globally, refused with `429` and a `Retry-After` header; raise it in
[`src/main.rs`](src/main.rs) if you drive more than one display.

## Attribution is mandatory

Both upstream sources are CC BY 4.0, so the `attribution` array is not decoration — if you
redistribute or display this data, you have to carry the credit through:

```json
[
  { "covers": ["place", "weather"], "source": "Open-Meteo",
    "license": "CC BY 4.0", "url": "https://open-meteo.com/" },
  { "covers": ["fuel"], "source": "Tankerkönig (MTS-K)",
    "license": "CC BY 4.0 - https://creativecommons.tankerkoenig.de",
    "url": "https://creativecommons.tankerkoenig.de/" }
]
```

Each entry names the top-level response fields it applies to, so a client that renders only
the weather knows which single credit it owes. Tankerkönig's notice is passed through
verbatim from their API rather than hardcoded. Their terms additionally require the credit
to be visible wherever prices are shown — plan a line for it in your display layout.

## Adapting to another region

- **Fuel prices** are the only Germany-specific part. Replace `get_fuel_prices` in
  [`src/provider.rs`](src/provider.rs) with an equivalent for your country, or drop it and
  leave `fuel` empty. The public `FuelStation` shape is generic enough to survive a
  different provider.
- **Language** — change `language: "de"` in the geocoder query to get place names in
  another language.
- **Search radius and forecast window** are hardcoded (7 km, 3 days) in
  [`src/lib.rs`](src/lib.rs) and [`src/provider.rs`](src/provider.rs).
- **Weather fields** are chosen by the `current`, `hourly` and `daily` query strings. Adding
  one means adding it to the wire type, the public type and the conversion.

## Data sources

- [Open-Meteo](https://open-meteo.com/) — geocoding and forecast. No key required.
- [Tankerkönig](https://creativecommons.tankerkoenig.de/) — German fuel prices from the
  MTS-K. Free key required, non-commercial use.

## License

MIT, see [LICENSE](LICENSE). This covers the code only, not the data from Open-Meteo and Tankerkönig.
