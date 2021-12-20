window.BENCHMARK_DATA = {
  "lastUpdate": 1640024307204,
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
      },
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
          "id": "54cdc051fe71eb76cef2a9a4c8c60ee2965888bc",
          "message": "Merge pull request #237 from zcash/orchard-mainnet-circuit\n\nOrchard proposed mainnet circuit",
          "timestamp": "2021-12-20T17:49:57Z",
          "tree_id": "22918a431caa24ea9345e7e9247945f06f8fa29a",
          "url": "https://github.com/zcash/orchard/commit/54cdc051fe71eb76cef2a9a4c8c60ee2965888bc"
        },
        "date": 1640024306169,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 7335945851,
            "range": "± 58813174",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 7299308297,
            "range": "± 53544201",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 10667157018,
            "range": "± 54257561",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 14070861458,
            "range": "± 315383463",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 39782495,
            "range": "± 1161194",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 39378255,
            "range": "± 1499350",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 44225140,
            "range": "± 1016817",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 49077721,
            "range": "± 1753320",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1257402,
            "range": "± 9018",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 159800,
            "range": "± 1430",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1251124,
            "range": "± 13776",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 163018139,
            "range": "± 1245828",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 24790561,
            "range": "± 88862",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2839568,
            "range": "± 15484",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 24722135,
            "range": "± 95694",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 11658575,
            "range": "± 59840",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 123505165,
            "range": "± 794407",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 14000795,
            "range": "± 117890",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 122647249,
            "range": "± 841639",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 57880307,
            "range": "± 492279",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 246853122,
            "range": "± 1325656",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 28115552,
            "range": "± 230056",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 246082804,
            "range": "± 1368643",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 115904911,
            "range": "± 961826",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 45229,
            "range": "± 404",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 166388,
            "range": "± 1709",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 181619,
            "range": "± 1414",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 299510,
            "range": "± 1471",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 298486,
            "range": "± 2889",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 169712,
            "range": "± 1749",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 183639,
            "range": "± 2937",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 301108,
            "range": "± 2291",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 299641,
            "range": "± 3713",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 353001,
            "range": "± 3618",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 369083,
            "range": "± 4015",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 487838,
            "range": "± 6347",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 487406,
            "range": "± 5306",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 597291,
            "range": "± 4080",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 677150,
            "range": "± 2048",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}