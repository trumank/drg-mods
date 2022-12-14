```
[GameState BeginPlay]  
[GameMode BeginPlay]

CD2 Core Locates Modules in Path  
CD2 Core Locates Mutators in Path

[PLS BeginPlay]  
CD2 Core Pauses PLS Init  
CD2 Core Begins Failsafe Timeout  
For each Module in the config:  
    Core Calls Load on the Module
    The Module Creates CD2 Values and binds them
        CD2 Values create Mutators
        Mutators create Mutators recursively
    <------ Errors are returned to Core
    If errors, break to PLS resume init
    CD2 Core Triggers Mutator Manager :: BeginCD2ValueProcessing
    Mutators Fire ready when they and their Children are Ready.
    Values fire OnChange when their root mutator is Ready
    Values fire OnReady after the first OnChange
    Module listens for all Value OnReady events
    Module fires OnEnable when all values are ready
    Module fires OnUpdate when all values are ready

CD2 Core listens for all modules to have fired OnLoad 
CD2 Core Destroys Failsafe Timeout  
CD2 Core Resumes PLS Init
```