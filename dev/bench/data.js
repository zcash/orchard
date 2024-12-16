window.BENCHMARK_DATA = {
  "lastUpdate": 1734388048061,
  "repoUrl": "https://github.com/zcash/orchard",
  "entries": {
    "Orchard Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "Jack Grigg",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d3df9c5be86a1441270a4e6a426f59a8890b70f4",
          "message": "Merge pull request #447 from zcash/keystone-prep\n\nKeystone preparations",
          "timestamp": "2024-12-17T11:13:24+13:00",
          "tree_id": "30db27d413ca72f8d3614860a4dc5d5f83b7d1ff",
          "url": "https://github.com/zcash/orchard/commit/d3df9c5be86a1441270a4e6a426f59a8890b70f4"
        },
        "date": 1734388047170,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2921709588,
            "range": "± 272435695",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2894628289,
            "range": "± 11151622",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4143757597,
            "range": "± 30515117",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5405695504,
            "range": "± 18725400",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 25146107,
            "range": "± 545429",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 25399787,
            "range": "± 643199",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28093473,
            "range": "± 480161",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31872682,
            "range": "± 898066",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1528794,
            "range": "± 6254",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 127135,
            "range": "± 237",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1526446,
            "range": "± 4613",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1334405303,
            "range": "± 2607680",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16137470,
            "range": "± 40836",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2160795,
            "range": "± 21059",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16121352,
            "range": "± 57824",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 14416325,
            "range": "± 54242",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 80673627,
            "range": "± 269275",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10750530,
            "range": "± 36949",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 80526392,
            "range": "± 618726",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 72029502,
            "range": "± 200541",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 161275512,
            "range": "± 257515",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21489589,
            "range": "± 72776",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 161012749,
            "range": "± 1873411",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 144040717,
            "range": "± 200882",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 465438,
            "range": "± 4772",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 502357,
            "range": "± 582",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}