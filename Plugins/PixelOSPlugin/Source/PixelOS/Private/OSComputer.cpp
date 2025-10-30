#include "OSComputer.h"
#include "PixelOSComponent.h"
#include "Materials/MaterialInstanceDynamic.h"
#include "Components/StaticMeshComponent.h"

AOSComputer::AOSComputer()
{
    PrimaryActorTick.bCanEverTick = true;

    ScreenMesh = CreateDefaultSubobject<UStaticMeshComponent>(TEXT("ScreenMesh"));
    RootComponent = ScreenMesh;

    OSComponent = CreateDefaultSubobject<UPixelOSComponent>(TEXT("OSComponent"));
}

void AOSComputer::BeginPlay()
{
    Super::BeginPlay();

    OSComponent->BootOS("PixelOS");

    if (OSComponent->OSDisplayTexture)
    {
        UMaterialInstanceDynamic* DynMaterial = UMaterialInstanceDynamic::Create(ScreenMesh->GetMaterial(0), this);
        DynMaterial->SetTextureParameterValue("ScreenTexture", OSComponent->OSDisplayTexture);
        ScreenMesh->SetMaterial(0, DynMaterial);
    }
}

void AOSComputer::Tick(float DeltaTime)
{
    Super::Tick(DeltaTime);
}
