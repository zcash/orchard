window.BENCHMARK_DATA = {
  "lastUpdate": 1651674517391,
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
          "id": "a30caec124aa6c6e7818b5100293204425c49de3",
          "message": "Merge pull request #314 from nuttycom/feature/spend_minconf\n\nUpdate incrementalmerkletree dependency version.",
          "timestamp": "2022-05-04T08:15:49-06:00",
          "tree_id": "70d178fdd1ed1a48611443b43a306ac195ae0627",
          "url": "https://github.com/zcash/orchard/commit/a30caec124aa6c6e7818b5100293204425c49de3"
        },
        "date": 1651674515738,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 4214830372,
            "range": "± 36988448",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 4213206608,
            "range": "± 15898106",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 6012926337,
            "range": "± 18414826",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 7813844456,
            "range": "± 22594992",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 33669269,
            "range": "± 1347486",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 33384827,
            "range": "± 250522",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 37610492,
            "range": "± 232266",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 41080665,
            "range": "± 1005871",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}