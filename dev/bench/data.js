window.BENCHMARK_DATA = {
  "lastUpdate": 1641393937708,
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
          "id": "dabf364b86958c82662317c0f2f50fbc104403fc",
          "message": "Merge pull request #268 from zcash/update-mockprover-errors\n\nUpdate `halo2` revision",
          "timestamp": "2022-01-05T14:15:42Z",
          "tree_id": "a1178f7e03759e7f3ca1bb5df330149cebe2f426",
          "url": "https://github.com/zcash/orchard/commit/dabf364b86958c82662317c0f2f50fbc104403fc"
        },
        "date": 1641393936292,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 7861651450,
            "range": "± 123501783",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 7839685218,
            "range": "± 79277426",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 11431479415,
            "range": "± 134737132",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 15046351305,
            "range": "± 132791582",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 41506556,
            "range": "± 1414616",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 41192330,
            "range": "± 834063",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 46672852,
            "range": "± 1885246",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 51290863,
            "range": "± 2274996",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1338509,
            "range": "± 30014",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 168941,
            "range": "± 4203",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1319810,
            "range": "± 46540",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 172850284,
            "range": "± 4481490",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 26238816,
            "range": "± 462022",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 3058851,
            "range": "± 65287",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 24945717,
            "range": "± 827431",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2953978,
            "range": "± 87366",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 131224807,
            "range": "± 2637736",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 14887916,
            "range": "± 475905",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 128952965,
            "range": "± 3139251",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 14866665,
            "range": "± 437540",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 264113596,
            "range": "± 3816587",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 29816083,
            "range": "± 1006687",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 257438642,
            "range": "± 5944333",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 28934337,
            "range": "± 883773",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 77684945,
            "range": "± 17388258",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4172727,
            "range": "± 169003",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 177811018,
            "range": "± 3781567",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 6160096,
            "range": "± 159339",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 254423613,
            "range": "± 6727717",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 7390103,
            "range": "± 327182",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 48201,
            "range": "± 2352",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 174696,
            "range": "± 5305",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 189223,
            "range": "± 8164",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 314517,
            "range": "± 8668",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 313459,
            "range": "± 15342",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 177346,
            "range": "± 5423",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 194344,
            "range": "± 69134",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 315451,
            "range": "± 14182",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 316222,
            "range": "± 12772",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 377248,
            "range": "± 8446",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 385949,
            "range": "± 13179",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 506096,
            "range": "± 19515",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 502007,
            "range": "± 18006",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 613203,
            "range": "± 31276",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 702075,
            "range": "± 30537",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}