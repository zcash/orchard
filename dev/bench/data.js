window.BENCHMARK_DATA = {
  "lastUpdate": 1640023079217,
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
          "id": "40cc3cb7281a3c50c06ba41b7ae79a8bc29ecc47",
          "message": "Merge pull request #267 from zcash/crate-cleanups\n\nCrate cleanups",
          "timestamp": "2021-12-20T17:35:53Z",
          "tree_id": "fec1f137f95fdeb96d2eefa8d475ce067a4320a1",
          "url": "https://github.com/zcash/orchard/commit/40cc3cb7281a3c50c06ba41b7ae79a8bc29ecc47"
        },
        "date": 1640023078274,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 6147692283,
            "range": "± 11178910",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 6160799568,
            "range": "± 12932903",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 8954576293,
            "range": "± 39893270",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 11822417801,
            "range": "± 23754042",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 33033016,
            "range": "± 178699",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 33059629,
            "range": "± 327704",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 37347853,
            "range": "± 287048",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 40943853,
            "range": "± 837989",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1052372,
            "range": "± 846",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 134351,
            "range": "± 614",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1049008,
            "range": "± 618",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 135950253,
            "range": "± 46603",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 20703044,
            "range": "± 8171",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2365831,
            "range": "± 12645",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 20654584,
            "range": "± 12463",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2329051,
            "range": "± 1432",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 103451604,
            "range": "± 39749",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 11773755,
            "range": "± 6373",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 103207102,
            "range": "± 48069",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 11591767,
            "range": "± 4924",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 206964323,
            "range": "± 81236",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 23523005,
            "range": "± 10525",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 206404692,
            "range": "± 51093",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 23156659,
            "range": "± 11539",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 37788,
            "range": "± 365",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 139705,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 152600,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 250698,
            "range": "± 202",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 250511,
            "range": "± 230",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 142519,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 155513,
            "range": "± 232",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 253521,
            "range": "± 124",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 253396,
            "range": "± 147",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 298352,
            "range": "± 1114",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 311104,
            "range": "± 216",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 410443,
            "range": "± 243",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 410427,
            "range": "± 146",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 498930,
            "range": "± 158",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 564174,
            "range": "± 219",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}