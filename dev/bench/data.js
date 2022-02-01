window.BENCHMARK_DATA = {
  "lastUpdate": 1643732666930,
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
          "id": "d270edaa14e0cf3d607a5ab12675800213b1c937",
          "message": "Merge pull request #274 from nuttycom/beta_lints\n\nUse beta lints instead of nightly.",
          "timestamp": "2022-02-01T16:01:57Z",
          "tree_id": "7dcc423b07b7be6e6fbb5c403045d3512ed91858",
          "url": "https://github.com/zcash/orchard/commit/d270edaa14e0cf3d607a5ab12675800213b1c937"
        },
        "date": 1643732666109,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 3869361483,
            "range": "± 30203263",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 3868856740,
            "range": "± 8328282",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 5547078549,
            "range": "± 18525961",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 7214576674,
            "range": "± 37834804",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 32359403,
            "range": "± 331002",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 32536881,
            "range": "± 662983",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 36406376,
            "range": "± 177597",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 40106969,
            "range": "± 543195",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1043757,
            "range": "± 3108",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 134020,
            "range": "± 97",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1048741,
            "range": "± 571",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 134097906,
            "range": "± 214665",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 20701743,
            "range": "± 20676",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2366959,
            "range": "± 1493",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 20647891,
            "range": "± 11340",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2330868,
            "range": "± 4684",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 103459959,
            "range": "± 47012",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 11779048,
            "range": "± 6655",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 103174097,
            "range": "± 35861",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 11598176,
            "range": "± 6116",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 206892739,
            "range": "± 47418",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 23547297,
            "range": "± 7475",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 206307942,
            "range": "± 208568",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 23180727,
            "range": "± 7439",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 62268069,
            "range": "± 11689607",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3300764,
            "range": "± 30938",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 149931188,
            "range": "± 753292",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4907791,
            "range": "± 33934",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 213959881,
            "range": "± 841096",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5932320,
            "range": "± 120296",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 39800,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 139011,
            "range": "± 160",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 151405,
            "range": "± 88",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 247494,
            "range": "± 117",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 247427,
            "range": "± 878",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 141955,
            "range": "± 100",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 154373,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 250396,
            "range": "± 1384",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 250526,
            "range": "± 190",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 296953,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 309367,
            "range": "± 3236",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 401854,
            "range": "± 253",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 405429,
            "range": "± 430",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 499385,
            "range": "± 350",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 565513,
            "range": "± 367",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}