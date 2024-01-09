window.BENCHMARK_DATA = {
  "lastUpdate": 1704841180665,
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
          "id": "9a85034ce932ca398da16529482e5efecc474c50",
          "message": "Merge pull request #412 from nuttycom/bundle_metadata\n\nReturn bundle metadata from bundle building.",
          "timestamp": "2024-01-09T15:45:01-07:00",
          "tree_id": "80277116568a826207188ce6d6b090b3c1a2b220",
          "url": "https://github.com/zcash/orchard/commit/9a85034ce932ca398da16529482e5efecc474c50"
        },
        "date": 1704841179623,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2906995820,
            "range": "± 177583410",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2904159248,
            "range": "± 13630176",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4153923763,
            "range": "± 21998777",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5395600825,
            "range": "± 13601940",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 25113967,
            "range": "± 580626",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 25351757,
            "range": "± 457009",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28232937,
            "range": "± 557916",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31693461,
            "range": "± 342514",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1526587,
            "range": "± 10148",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 127507,
            "range": "± 385",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1524724,
            "range": "± 4887",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1356675155,
            "range": "± 1993417",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16108805,
            "range": "± 102294",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2167969,
            "range": "± 7587",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16083923,
            "range": "± 105575",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2128172,
            "range": "± 8841",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 80548033,
            "range": "± 302027",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10814002,
            "range": "± 34137",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 80324921,
            "range": "± 200742",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10602115,
            "range": "± 42454",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 161001544,
            "range": "± 191147",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21604694,
            "range": "± 43968",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 160620423,
            "range": "± 395421",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 21152306,
            "range": "± 73861",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 466672,
            "range": "± 1089",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 500695,
            "range": "± 7837",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}