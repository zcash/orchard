window.BENCHMARK_DATA = {
  "lastUpdate": 1723491649898,
  "repoUrl": "https://github.com/zcash/orchard",
  "entries": {
    "Orchard Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "kris@nutty.land",
            "name": "Kris Nuttycombe",
            "username": "nuttycom"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5db68d004cbc30f13b153beb77b2eb665d34fa75",
          "message": "Merge pull request #434 from nuttycom/release/orchard-0.9.0\n\nRelease orchard version 0.9.0",
          "timestamp": "2024-08-12T13:25:38-06:00",
          "tree_id": "f2d655bb96e842d41ad8c3aa3100c40d3eea460c",
          "url": "https://github.com/zcash/orchard/commit/5db68d004cbc30f13b153beb77b2eb665d34fa75"
        },
        "date": 1723491648413,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2912425178,
            "range": "± 49973728",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2928642171,
            "range": "± 9909786",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4138605314,
            "range": "± 17162619",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5432066575,
            "range": "± 24921723",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 25344714,
            "range": "± 533863",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 25186180,
            "range": "± 626437",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28357593,
            "range": "± 600545",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31628214,
            "range": "± 432255",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1523196,
            "range": "± 37805",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 127596,
            "range": "± 551",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1521341,
            "range": "± 10574",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1338118251,
            "range": "± 5229702",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16083368,
            "range": "± 53483",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2167298,
            "range": "± 4005",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16069457,
            "range": "± 13921",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2132401,
            "range": "± 13297",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 80361092,
            "range": "± 1492215",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10785007,
            "range": "± 42632",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 80278130,
            "range": "± 381063",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10607135,
            "range": "± 33717",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 160715060,
            "range": "± 1959578",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21547094,
            "range": "± 64036",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 160490289,
            "range": "± 294894",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 21199131,
            "range": "± 78419",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 467554,
            "range": "± 1214",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 502672,
            "range": "± 577",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}