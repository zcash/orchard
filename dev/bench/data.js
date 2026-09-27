window.BENCHMARK_DATA = {
  "lastUpdate": 1790479878029,
  "repoUrl": "https://github.com/zcash/orchard",
  "entries": {
    "Orchard Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "jack@zodl.com",
            "name": "Danny Willems",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "dea3ad6d3d8b6b56698893d75d1f49fdefcb5da4",
          "message": "Merge pull request #554 from zcash/dw/ff-rand-group\n\nMigrate to ff 0.14, group 0.14 and rand 0.10",
          "timestamp": "2026-09-27T04:19:53+01:00",
          "tree_id": "1cf060581b5e42cda5e92029cdbe73da07d220c6",
          "url": "https://github.com/zcash/orchard/commit/dea3ad6d3d8b6b56698893d75d1f49fdefcb5da4"
        },
        "date": 1790479877012,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2509172971,
            "range": "± 20528119",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2507405565,
            "range": "± 12587389",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 3609303360,
            "range": "± 32832297",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 4679022665,
            "range": "± 9905662",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 19352131,
            "range": "± 140418",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 19288090,
            "range": "± 479217",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 22421019,
            "range": "± 139506",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 25156262,
            "range": "± 198765",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1251987,
            "range": "± 14093",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 103473,
            "range": "± 165",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1251023,
            "range": "± 4514",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1110602427,
            "range": "± 761718",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 12614837,
            "range": "± 72395",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 1119215,
            "range": "± 5774",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 12590638,
            "range": "± 25886",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 1084939,
            "range": "± 13271",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 62997771,
            "range": "± 125598",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 5523595,
            "range": "± 9652",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 62882598,
            "range": "± 201327",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 5339398,
            "range": "± 91726",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 125898143,
            "range": "± 176451",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 11017552,
            "range": "± 27442",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 125635082,
            "range": "± 218081",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 10662046,
            "range": "± 40045",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 380855,
            "range": "± 1485",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 410058,
            "range": "± 1328",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}