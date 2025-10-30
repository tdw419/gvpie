#pragma once

#include "CoreMinimal.h"
#include "GameFramework/Actor.h"
#include "OSComputer.generated.h"

class UPixelOSComponent;

UCLASS()
class PIXELOS_API AOSComputer : public AActor
{
    GENERATED_BODY()

public:
    AOSComputer();

protected:
    virtual void BeginPlay() override;

public:
    virtual void Tick(float DeltaTime) override;

    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Components")
    UStaticMeshComponent* ScreenMesh;

    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Components")
    UPixelOSComponent* OSComponent;
};
