window.BENCHMARK_DATA = {
  "lastUpdate": 1643850365489,
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
          "id": "1f4acdf878def4999f9ab3da3491290f350a5fc8",
          "message": "Merge pull request #270 from zcash/derive-internal-keys\n\nDerive internal keys.",
          "timestamp": "2022-02-03T00:42:23Z",
          "tree_id": "18b59c85548e7fd44d8b54675ebff1268283a440",
          "url": "https://github.com/zcash/orchard/commit/1f4acdf878def4999f9ab3da3491290f350a5fc8"
        },
        "date": 1643850364502,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 4475459011,
            "range": "± 96336650",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 4376705104,
            "range": "± 99667593",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 6066772467,
            "range": "± 70707770",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 7697955356,
            "range": "± 81028403",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 35739587,
            "range": "± 1595229",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 35622886,
            "range": "± 2784638",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 39186463,
            "range": "± 1444228",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 43551293,
            "range": "± 1811202",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1093292,
            "range": "± 42016",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 136748,
            "range": "± 8740",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1064671,
            "range": "± 63400",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 158047963,
            "range": "± 2514293",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 22672300,
            "range": "± 566219",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2361340,
            "range": "± 92004",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 21465274,
            "range": "± 897313",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2599107,
            "range": "± 102989",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 113043279,
            "range": "± 6022483",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 13982386,
            "range": "± 225883",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 117121152,
            "range": "± 5983964",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 13183734,
            "range": "± 340717",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 233312045,
            "range": "± 7718407",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 26994318,
            "range": "± 740341",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 244750141,
            "range": "± 8584275",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 26711666,
            "range": "± 792919",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 71141787,
            "range": "± 3518340",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3743903,
            "range": "± 121483",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 170582035,
            "range": "± 4372156",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 5594038,
            "range": "± 94033",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 244899607,
            "range": "± 5696143",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 6736162,
            "range": "± 176241",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 45427,
            "range": "± 865",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 164463,
            "range": "± 3508",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 179546,
            "range": "± 6082",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 295701,
            "range": "± 6759",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 296042,
            "range": "± 6872",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 154679,
            "range": "± 10235",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 176673,
            "range": "± 6596",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 287087,
            "range": "± 8394",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 274349,
            "range": "± 16379",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 330585,
            "range": "± 17809",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 347634,
            "range": "± 15350",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 467910,
            "range": "± 14330",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 464703,
            "range": "± 17341",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 538417,
            "range": "± 20516",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 616699,
            "range": "± 24330",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}