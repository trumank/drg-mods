## ByMissionType
Change the value based on mission type. If a value isn't set for a mission type it will use the 'Default' value. A 'Default' must be set unless a value is specified for all mission types.

###  Supported mission types
* Egg
* Elimination
* Escort
* Mining
* PE
* Refinery
* Sabotage
* Salvage

### Example
```
{
    "Mutate": "ByMissionType",
    "Default": 60,
    "Egg": 70,
    "PE": 80
 }
```

----------
## ByBiome
Change the value based on the mission biome. If a value isn't set for a biome it will use the 'Default' value. A 'Default' must be set unless a value is specified for biomes. If a biome is not recognized and no default is specified, the value for CrystallineCaverns will be used.

###  Biome Names and Aliases
* BIOME_AzureWeald -- "AzureWeald"
* BIOME_CrystalCaves -- "CrystalCave" or "CrystallineCaverns"
* BIOME_FungusBogs -- "FungusBogs"
* BIOME_HollowBough -- "HollowBough"
* BIOME_IceCaves -- "IceCaves" or "GlacialStrata"
* BIOME_MagmaCaves -- "MagmaCaves" or "MagmaCore"
* BIOME_SandblastedCorridors -- "SandblastedCorridors" or "Sandblasted"
* BIOME_RadioactiveZone -- "RadioactiveExclusionZone" or "RadioactiveZone" or "REZ"
* BIOME_SaltCaves -- "SaltCaves" or "SaltPits"

### Example
```
{
    "Mutate": "ByBiome",
    "Default": false,
    "SaltPits": true,
    "REZ": true
 }
```

----------

## ByPlayerCount
Change the value based on the number of players in game. A solo game gets the first position in the list; two players get the second spot and so on. The last value in the list is used if there are more players than there are values in the list.

### Example
```
{
    "Mutate": "ByPlayerCount",
    "Values": [
        80,
        120,
        180,
        180
    ]
}
```

### *Note on Implicit Player Count Control*
*Values that expect a float can be automatically controlled by player count by placing an array as the value. e.g. `"Resupply": {"Cost": [80, 60, 40]}` would set the cost of resupplies to 80 in solo, 60 in duo, and 40 three or more players.*

----------

## ByTime
Change a value over time. Time matches the mission clock in the escape menu in-game, including starting before players receive control of their dwarves.

###  Parameters
* `InitialValue` - Value at time 0 and up until `StartDelay`
* `StartDelay` - Time in seconds to stay at the `InitialValue` before changing.
* `RateOfChange` - Rate per second to change the value. Value is `InitialValue + RateOfChange * Max(0, Time - StartDelay)`

### Example
```
{
    "Mutate": "ByTime",
    "InitialValue": 3.1,
    "RateOfChange": 0.0033,
    "StartDelay": 400
 }
```

----------

## Clamp
Constrain a float (number) to fall within a range. This range is inclusive. If only a min or only a max is specified, the value will only be clamped in that direction.

### Example
```
{
    "Mutate": "Clamp",
    "Value": 90,
    "Min": 0,
    "Max": 100
}
```

----------

## IfFloat
Choose one value or another based on a comparison of two float values.

The two floats are specified by 'Value' and the operator to be used e.g. '>=' if the comparison was to be greater than or equal. If the condition is true the 'Then' value is used, else the 'Else' condition is used.

Valid operators are: `==`, `>=`, `>`, `<=`, `<`

### Example
*In this example, as long as the team has called less than 2 resupplies the value is 40. After the team has called their second resupply, the value is 80.*
```
{
    "Mutate": "IfFloat",
    "Value": {"Mutator": "ResuppliesCalled"},
    "<": 2
    "Then": 40,
    "Else": 80
}
```

----------

## RandomChoicePerMission
Choose one of a set of values for each mission. The choice is fixed to the seed of the mission. Subsequent plays of the same seed will use the same value. This mutator has a single property, 'Choices' an array of values to choose from.

### Example
```
{
    "Mutate": "RandomChoicePerMission",
    "Choices": [
        "bedrock",
        "hotrock",
        "dirt"
        ]
}
```

----------



# Player Monitor
Utilities for reacting to team status.

## DwarvesDown (Value)
Float count of the dwarves that are currently down, 0 if no dwarves are currently downed, 4 if all 4 dwarves are down.

### Example
```
{
    "Mutate": "DwarvesDown"
}
```

----------

## IWsLeft (Value)
Float count of the number of IWs the team still has in reserve, 0 when no IWs remain. *This will be 0 until dwarves are spawned which happens after the level is setup including encounters and terrain.*

### Example
```
{
    "Mutate": "IWsLeft"
}
```

----------

## DwarvesAmmo (Value)
Average percent ammo left for the team, 1 when all teammates have 100% of their ammo and 0 when all teammates are at 0% ammo. This works the same way as the 4 bars under the dwarves names in the UI.

### Example
```
{
    "Mutate": "DwarvesAmmo"
}
```

----------

## DwarvesHealth (Value)
Average health, 1 when all teammates are at 100% health and 0 when all teammates are down. 

### Example
```
{
    "Mutate": "DwarvesHealth"
}
```

----------

## DwarvesShield (Value)
Average sheilds, 1 when all teammates are at full shield 0 when all teammates are at 0 shield. Untested on shield disruption. 

### Example
```
{
    "Mutate": "DwarvesShield"
}
```

----------

## DwarvesDowns (Value)
Total number of downs for the team during the mission. This might not match the end screen because it will still count downs from disconnected players, and will not over-count downs.

### Example
```
{
    "Mutate": "DwarvesDowns"
}
```

----------

## DwarvesRevives (Value)
Total number of revives for the team during the mission. This includes IW self-revives.

### Example
```
{
    "Mutate": "DwarvesRevives"
}
```

----------

## DwarvesDownTime (Value)
Time in seconds that has elapsed while a dwarf has been down. If multiple dwarves are down, it's the longest time down among the downed dwarves.

### Example
```
{
    "Mutate": "DwarvesDownTime"
}
```

----------


# Resupply Monitor
Utilities for reacting to resupply pod use.

## ResuppliesCalled (Value)
Number of resupplies the team has called during the mission. This should increment almost immediately after a resupply is initiated, before another resupply can be called.

### Example
```
{
    "Mutate": "ResuppliesCalled"
}
```

----------

## ResupplyUsesLeft (Value)
The count among all supply pods of uses left on a resupply. By default, resupplies start with 4 uses.

### Example
```
{
    "Mutate": "ResupplyUsesLeft"
}
```

----------

## ResupplyUsesConsumed (Value)
The number of uses that have been consumed from resupplies this mission.

### Example
```
{
    "Mutate": "ResupplyUsesConsumed"
}
```

----------

# Depository Monitor
Utilities for reacting to held and deposited team resources.

## DepositedResource (Value)
Count of a resource that has been deposited into the team depository. Any resource can be referenced by the name as it appears in the in-game UI.

### Example
```
{
    "Mutate": "DepositedResource"
    "Resource": "Nitra"
}
```

----------

## HeldResource (Value)
For a resource, the sum of that resource in players' inventories, not desposited.

### Example
```
{
    "Mutate": "HeldResource"
    "Resource": "Apoca Bloom"
}
```

----------

## TotalResource (Value)
Sum of a resource held in players inventories and the group depot. Any resource can be referenced by the name as it appears in the in-game UI.

### Example
```
{
    "Mutate": "TotalResource"
    "Resource": "Morkite"
}
```

----------
