window.BENCHMARK_DATA = {
  "lastUpdate": 1707255782935,
  "repoUrl": "https://github.com/zcash/orchard",
  "entries": {
    "Orchard Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "daira@jacaranda.org",
            "name": "Daira-Emma Hopwood",
            "username": "daira"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3d79ba47fe3b3a70632e63feefe6368bb35c5d21",
          "message": "Merge pull request #417 from daira/improve-doc-for-fixed-base-constants\n\nImprove documentation for fixed-base constants",
          "timestamp": "2024-02-06T21:27:24Z",
          "tree_id": "c5a438972eb155267cd7657a260d8a16806ad893",
          "url": "https://github.com/zcash/orchard/commit/3d79ba47fe3b3a70632e63feefe6368bb35c5d21"
        },
        "date": 1707255781616,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2925705155,
            "range": "± 169478400",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2927364645,
            "range": "± 20713920",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4158521465,
            "range": "± 18515788",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5397017910,
            "range": "± 28475611",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 25261724,
            "range": "± 488276",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 25283181,
            "range": "± 527626",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28187682,
            "range": "± 811926",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31823474,
            "range": "± 466263",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1526462,
            "range": "± 9371",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 127530,
            "range": "± 423",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1523517,
            "range": "± 4232",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1343902777,
            "range": "± 1635834",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16120959,
            "range": "± 86456",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2167850,
            "range": "± 3842",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16087556,
            "range": "± 103988",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2141371,
            "range": "± 3715",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 80507994,
            "range": "± 319882",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10783447,
            "range": "± 41926",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 80374260,
            "range": "± 181856",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10671178,
            "range": "± 60186",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 161043290,
            "range": "± 315949",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21543982,
            "range": "± 623180",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 160766284,
            "range": "± 296071",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 21332856,
            "range": "± 61509",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 465068,
            "range": "± 2060",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 501338,
            "range": "± 10842",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}