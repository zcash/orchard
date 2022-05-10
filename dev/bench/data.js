window.BENCHMARK_DATA = {
  "lastUpdate": 1652221692239,
  "repoUrl": "https://github.com/zcash/orchard",
  "entries": {
    "Orchard Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "str4d",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "43ec9118413569ce8e708bff25ff912669d87f29",
          "message": "Merge pull request #327 from zcash/halo2_gadgets-api-changes\n\nMigrate to final `halo2_gadgets` pre-release revision",
          "timestamp": "2022-05-10T23:11:29+01:00",
          "tree_id": "540ed99c8d4a1b69efacdf41350ec560dbe2a7ee",
          "url": "https://github.com/zcash/orchard/commit/43ec9118413569ce8e708bff25ff912669d87f29"
        },
        "date": 1652221691046,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 3860236728,
            "range": "± 39871583",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 3828459994,
            "range": "± 6356279",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 5470655751,
            "range": "± 14037165",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 7132289144,
            "range": "± 20410385",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 32940127,
            "range": "± 1001660",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 33256828,
            "range": "± 282888",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 37226482,
            "range": "± 925519",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 40775656,
            "range": "± 322576",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1047430,
            "range": "± 1376",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 132640,
            "range": "± 178",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1044932,
            "range": "± 2314",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 134189275,
            "range": "± 30414",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 20616692,
            "range": "± 9503",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2320550,
            "range": "± 913",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 20568742,
            "range": "± 36287",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2302736,
            "range": "± 974",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 103032366,
            "range": "± 39177",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 11642590,
            "range": "± 4812",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 102783962,
            "range": "± 29137",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 11460289,
            "range": "± 3763",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 206082506,
            "range": "± 480373",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 23254125,
            "range": "± 7323",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 205546530,
            "range": "± 39506",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 22908975,
            "range": "± 102167",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 494498,
            "range": "± 248",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 546277,
            "range": "± 348",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}