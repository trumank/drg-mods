#pragma once
#include "CoreMinimal.h"
#include "UObject/NoExportTypes.h"
#include "EDeepMovementState.h"
#include "DeepRepPath.generated.h"

USTRUCT(BlueprintType)
struct FDeepRepPath {
    GENERATED_BODY()
public:
    UPROPERTY(BlueprintReadWrite, EditAnywhere, Transient, meta=(AllowPrivateAccess=true))
    FVector PathBase;
    
    UPROPERTY(BlueprintReadWrite, EditAnywhere, Transient)
    uint8 PathLength;
    
    UPROPERTY(BlueprintReadWrite, EditAnywhere, Transient, meta=(AllowPrivateAccess=true))
    EDeepMovementState State;
    
    UPROPERTY(BlueprintReadWrite, EditAnywhere, Transient)
    uint8 StateBits;
    
    UPROPERTY(BlueprintReadWrite, EditAnywhere, Transient)
    TArray<FVector> PathOffsets;
    // FVector PathOffsets[16];
    
    FSD_API FDeepRepPath();
};

