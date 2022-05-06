window.BENCHMARK_DATA = {
  "lastUpdate": 1651848734931,
  "repoUrl": "https://github.com/zcash/orchard",
  "entries": {
    "Orchard Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "yingtong@z.cash",
            "name": "ying tong",
            "username": "therealyingtong"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8f5ac0f36c2aa0b61bb6bedc1b997e641358a122",
          "message": "Merge pull request #324 from zcash/fix-reddsa-patch-rev\n\nFix patch revision for `reddsa` crate",
          "timestamp": "2022-05-06T22:39:51+08:00",
          "tree_id": "4932afa8b8a88896a5517c32be5044aa6d424b9d",
          "url": "https://github.com/zcash/orchard/commit/8f5ac0f36c2aa0b61bb6bedc1b997e641358a122"
        },
        "date": 1651848733855,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 4180426351,
            "range": "± 41120057",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 4169514253,
            "range": "± 15975286",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 5962979265,
            "range": "± 18885501",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 7800824923,
            "range": "± 28324631",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 32911920,
            "range": "± 222682",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 33137716,
            "range": "± 283572",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 36957457,
            "range": "± 432404",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 40579056,
            "range": "± 300152",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}