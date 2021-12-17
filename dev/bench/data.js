window.BENCHMARK_DATA = {
  "lastUpdate": 1639781757390,
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
          "id": "a061a861b3e01cf2fa14e9f8b93d8922f7cc6de8",
          "message": "Merge pull request #266 from zcash/release-0.1.0-beta.1\n\nRelease 0.1.0-beta.1",
          "timestamp": "2021-12-17T22:30:14Z",
          "tree_id": "b230a23419d493fa1c517677f26067c1ec6d310e",
          "url": "https://github.com/zcash/orchard/commit/a061a861b3e01cf2fa14e9f8b93d8922f7cc6de8"
        },
        "date": 1639781756452,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 7519505742,
            "range": "± 177278380",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 7498469218,
            "range": "± 115596264",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 10970385010,
            "range": "± 228111689",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 14618428606,
            "range": "± 268678355",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 37853713,
            "range": "± 2234688",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 39772048,
            "range": "± 2831460",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 45609437,
            "range": "± 1772075",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 49117511,
            "range": "± 3146601",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1423027,
            "range": "± 41352",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 182232,
            "range": "± 4340",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1154511,
            "range": "± 77481",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 168216699,
            "range": "± 5315213",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 27786289,
            "range": "± 1042981",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 3044941,
            "range": "± 130362",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 23974111,
            "range": "± 1425109",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2814380,
            "range": "± 105692",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 138960150,
            "range": "± 3108233",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 14578634,
            "range": "± 644857",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 122989867,
            "range": "± 6402509",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 14264546,
            "range": "± 616909",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 277779147,
            "range": "± 5856615",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 28901117,
            "range": "± 1507503",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 242670290,
            "range": "± 10399465",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 27417820,
            "range": "± 2168982",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 45994,
            "range": "± 3316",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 164451,
            "range": "± 11536",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 169084,
            "range": "± 13291",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 281767,
            "range": "± 24878",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 299659,
            "range": "± 13443",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 167445,
            "range": "± 8361",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 172647,
            "range": "± 11206",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 302860,
            "range": "± 19078",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 304751,
            "range": "± 18001",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 347067,
            "range": "± 17802",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 365246,
            "range": "± 24587",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 475851,
            "range": "± 30799",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 470813,
            "range": "± 42363",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 591350,
            "range": "± 28335",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 666715,
            "range": "± 37600",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}