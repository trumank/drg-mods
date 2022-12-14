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
    "Mutator": "ByMissionType",
    "Default": 60,
    "Egg": 70,
    "PE": 80
 }
```

----------

## ByPlayerCount
Change the value based on the number of players in game. A solo game gets the first position in the list; two players get the second spot and so on. The last value in the list is used if there are more players than there are values in the list.

### Example
```
{
    "Mutator": "ByPlayerCount",
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

## Clamp
Constrain a float (number) to fall within a range. This range is inclusive. If only a min or only a max is specified, the value will only be clamped in that direction.

### Example
```
{
    "Mutator": "Clamp",
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
    "Mutator": "IfFloat",
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
    "Mutator": "RandomChoicePerMission",
    "Choices": [
        "bedrock",
        "hotrock",
        "dirt"
        ]
}
```