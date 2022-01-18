window.BENCHMARK_DATA = {
  "lastUpdate": 1642519087875,
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
          "id": "3e0449ed35205c1d558d2a4f2200a7e4334f3e99",
          "message": "Merge pull request #271 from zcash/tests-pasta-prep\n\nMigrate tests from `FieldExt::rand` to `Field::random`",
          "timestamp": "2022-01-18T14:48:07Z",
          "tree_id": "bfdb6a2a62e55372cdfd93b449ae6489c0bf4d34",
          "url": "https://github.com/zcash/orchard/commit/3e0449ed35205c1d558d2a4f2200a7e4334f3e99"
        },
        "date": 1642519086424,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 7756331913,
            "range": "± 72574372",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 7722480149,
            "range": "± 103998028",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 11238417176,
            "range": "± 135192540",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 14738930559,
            "range": "± 168382978",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 40809660,
            "range": "± 1281028",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 41044158,
            "range": "± 1666396",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 45649439,
            "range": "± 2786508",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 51528425,
            "range": "± 3027350",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1328314,
            "range": "± 39909",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 168338,
            "range": "± 5559",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1267857,
            "range": "± 61880",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 161723270,
            "range": "± 5405881",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 26453266,
            "range": "± 1027355",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2975012,
            "range": "± 143122",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 24936205,
            "range": "± 1140244",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2845445,
            "range": "± 111952",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 131110660,
            "range": "± 3094135",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 14682067,
            "range": "± 646086",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 124264240,
            "range": "± 4437876",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 14375733,
            "range": "± 756774",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 262138438,
            "range": "± 4376049",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 28994666,
            "range": "± 1019084",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 248877933,
            "range": "± 6782482",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 28587228,
            "range": "± 1146941",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 77152177,
            "range": "± 17702908",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4090130,
            "range": "± 174971",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 173993131,
            "range": "± 5550800",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 6062641,
            "range": "± 546353",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 249047075,
            "range": "± 10792234",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 7350422,
            "range": "± 342062",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 49194,
            "range": "± 2767",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 166652,
            "range": "± 6981",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 180658,
            "range": "± 7701",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 303453,
            "range": "± 11313",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 300781,
            "range": "± 16326",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 167416,
            "range": "± 6982",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 185783,
            "range": "± 8208",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 304829,
            "range": "± 16319",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 302032,
            "range": "± 12817",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 356444,
            "range": "± 12467",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 370877,
            "range": "± 17939",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 494239,
            "range": "± 18133",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 492324,
            "range": "± 21070",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 616356,
            "range": "± 25412",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 676743,
            "range": "± 53492",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}