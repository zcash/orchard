window.BENCHMARK_DATA = {
  "lastUpdate": 1639669314083,
  "repoUrl": "https://github.com/zcash/orchard",
  "entries": {
    "Orchard Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "ewillbefull@gmail.com",
            "name": "ebfull",
            "username": "ebfull"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4592c2f2754344ba60b559f9a41d77ca534572ef",
          "message": "Merge pull request #262 from zcash/261-ak_P-reject-identity\n\nReject the identity in `SpendValidatingKey::from_bytes`",
          "timestamp": "2021-12-16T08:19:58-07:00",
          "tree_id": "b7367ec08a57d797cf4d6d11bbd3436c491ad037",
          "url": "https://github.com/zcash/orchard/commit/4592c2f2754344ba60b559f9a41d77ca534572ef"
        },
        "date": 1639669312993,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 6142891847,
            "range": "± 15252054",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 6138730180,
            "range": "± 27429156",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 8936242390,
            "range": "± 24053176",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 11763368701,
            "range": "± 18961227",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 32912832,
            "range": "± 9113611",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 32715223,
            "range": "± 242242",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 36463889,
            "range": "± 225142",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 41026808,
            "range": "± 738657",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1046121,
            "range": "± 768",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 134104,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1051633,
            "range": "± 1147",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 137215598,
            "range": "± 42627",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 20756803,
            "range": "± 7207",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2363570,
            "range": "± 1207",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 20715235,
            "range": "± 13155",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2326607,
            "range": "± 1151",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 103733856,
            "range": "± 46356",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 11761647,
            "range": "± 4678",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 103497970,
            "range": "± 33449",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 11578404,
            "range": "± 39802",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 207459561,
            "range": "± 78148",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 23489302,
            "range": "± 7235",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 206953923,
            "range": "± 89930",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 23128292,
            "range": "± 74936",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 39748,
            "range": "± 143",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 139516,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 152051,
            "range": "± 250",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 248563,
            "range": "± 97",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 248643,
            "range": "± 110",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 142498,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 154934,
            "range": "± 94",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 249056,
            "range": "± 204",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 251518,
            "range": "± 250",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 298100,
            "range": "± 210",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 310529,
            "range": "± 194",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 406981,
            "range": "± 178",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 407112,
            "range": "± 142",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 498749,
            "range": "± 176",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 564553,
            "range": "± 570",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}