# est-tbp

read the pdf or something idk

`cargo t` tests are for sanity checking my probability functions

`cargo r` to run the basic analysis

## example output

filter: score > 35 (based on cv)

weights: {cr: 1, cd: 1, spd: 1, atk: 0.75} * 6.48
```
=====================================================        
params: Head, Hp                                             
p_main = 12.50%
p_sub  = 6.81%
p      = 0.85%

est. relic count = 117.5
        est. tbp = 2237.6 (9.3 days)
=====================================================
params: Hands, Atk

p_main = 12.50%
p_sub  = 2.82%
p      = 0.35%

est. relic count = 283.8
        est. tbp = 5404.9 (22.5 days)
=====================================================
params: Body, CritRate

p_main = 1.25%
p_sub  = 2.76%
p      = 0.03%

est. relic count = 2902.1
        est. tbp = 55278.1 (230.3 days)
=====================================================
params: Feet, Spd

p_main = 1.25%
p_sub  = 3.40%
p      = 0.04%

est. relic count = 2353.3
        est. tbp = 44824.6 (186.8 days)
=====================================================
params: Orb, IceDmgBoost

p_main = 2.32%
p_sub  = 5.37%
p      = 0.12%

est. relic count = 802.9
        est. tbp = 15293.2 (63.7 days)
=====================================================
params: Rope, AtkPercent

p_main = 6.67%
p_sub  = 6.81%
p      = 0.45%

est. relic count = 220.3
        est. tbp = 4195.6 (17.5 days)
```