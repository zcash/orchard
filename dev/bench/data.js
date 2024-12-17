window.BENCHMARK_DATA = {
  "lastUpdate": 1734411246947,
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
          "id": "953a4b615501ffe6884969b9359d262849b55569",
          "message": "Merge pull request #448 from zcash/release/0.10.1\n\nRelease orchard version 0.10.1",
          "timestamp": "2024-12-17T17:40:15+13:00",
          "tree_id": "ae46aa850d7c4f406bc2294d3193e139cdcdc6b2",
          "url": "https://github.com/zcash/orchard/commit/953a4b615501ffe6884969b9359d262849b55569"
        },
        "date": 1734411246034,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2903881731,
            "range": "± 26927635",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2898128625,
            "range": "± 7909339",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4179192865,
            "range": "± 28105752",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5426507723,
            "range": "± 19005719",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 25255917,
            "range": "± 555829",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 24880907,
            "range": "± 636546",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28135174,
            "range": "± 1209578",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31341970,
            "range": "± 429577",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1515917,
            "range": "± 5152",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 126908,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1513513,
            "range": "± 9505",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1348188971,
            "range": "± 1494307",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16000224,
            "range": "± 35012",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2161773,
            "range": "± 17980",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 15982015,
            "range": "± 39854",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2120118,
            "range": "± 7897",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 79966040,
            "range": "± 282789",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10759057,
            "range": "± 22490",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 79826988,
            "range": "± 358423",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10554918,
            "range": "± 184922",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 159910980,
            "range": "± 933302",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21501041,
            "range": "± 50721",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 159650123,
            "range": "± 328509",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 21094715,
            "range": "± 38265",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 465407,
            "range": "± 899",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 501715,
            "range": "± 3826",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}