window.BENCHMARK_DATA = {
  "lastUpdate": 1651251263424,
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
          "id": "68882c72d205e03706247ebc3acf17cfd94753d1",
          "message": "Merge pull request #317 from zcash/71-rename-bundle-authorization\n\nRename `Bundle::{try_}authorize` to `Bundle::{try_}map_authorization`",
          "timestamp": "2022-04-29T10:38:22-06:00",
          "tree_id": "af552d25f8c35f0e11505bb8223977f4e9be2ab1",
          "url": "https://github.com/zcash/orchard/commit/68882c72d205e03706247ebc3acf17cfd94753d1"
        },
        "date": 1651251261875,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 3778496433,
            "range": "± 118087067",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 3861947144,
            "range": "± 198717357",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 5379586468,
            "range": "± 125510126",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 6912479504,
            "range": "± 219733682",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 32824286,
            "range": "± 1375968",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 30261705,
            "range": "± 1838826",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 32984062,
            "range": "± 1495591",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 36406778,
            "range": "± 2064030",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 918163,
            "range": "± 10293",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 117133,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 921268,
            "range": "± 464",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 118767497,
            "range": "± 51629",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 18212249,
            "range": "± 11885",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2064058,
            "range": "± 1144",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 18119430,
            "range": "± 9808",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2031545,
            "range": "± 1998",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 90414630,
            "range": "± 31514",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10273760,
            "range": "± 44764",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 90552035,
            "range": "± 42340",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10110065,
            "range": "± 3887",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 182064076,
            "range": "± 305526",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 20533908,
            "range": "± 63482",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 181115350,
            "range": "± 4913990",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 20202165,
            "range": "± 10994",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 435482,
            "range": "± 151",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 481981,
            "range": "± 360",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}