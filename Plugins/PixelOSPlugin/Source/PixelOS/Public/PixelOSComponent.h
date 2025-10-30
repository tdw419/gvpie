#pragma once

#include "CoreMinimal.h"
#include "Components/ActorComponent.h"
#include "PixelOSComponent.generated.h"

// FFI function pointers
typedef void* (*pixelos_create_t)(struct OSConfig);
typedef void (*pixelos_destroy_t)(void*);
typedef void (*pixelos_step_t)(void*);
typedef void (*pixelos_send_key_t)(void*, uint32, uint32);
typedef const uint8* (*pixelos_get_framebuffer_t)(void*);

UCLASS(ClassGroup=(Custom), meta=(BlueprintSpawnableComponent))
class PIXELOS_API UPixelOSComponent : public UActorComponent
{
    GENERATED_BODY()

public:
    UPixelOSComponent();

protected:
    virtual void BeginPlay() override;
    virtual void EndPlay(const EEndPlayReason::Type EndPlayReason) override;

public:
    virtual void TickComponent(float DeltaTime, ELevelTick TickType, FActorComponentTickFunction* ThisTickFunction) override;

    UFUNCTION(BlueprintCallable, Category = "PixelOS")
    void BootOS(FString OSName);

    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "PixelOS")
    UTexture2D* OSDisplayTexture;

private:
    void* FFIHandle;
    void* OSInstance;

    pixelos_create_t pixelos_create;
    pixelos_destroy_t pixelos_destroy;
    pixelos_step_t pixelos_step;
    pixelos_send_key_t pixelos_send_key;
    pixelos_get_framebuffer_t pixelos_get_framebuffer;

    void LoadFFILibrary();
    void UnloadFFILibrary();
    void UpdateTexture();
};

struct OSConfig {
    uint32 width;
    uint-32 height;
};
