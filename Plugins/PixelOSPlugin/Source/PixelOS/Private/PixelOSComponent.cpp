#include "PixelOSComponent.h"
#include "Engine/Texture2D.h"
#include "Runtime/Engine/Classes/Engine/Texture2D.h"
#include "GenericPlatform/GenericPlatformProcess.h"

UPixelOSComponent::UPixelOSComponent()
{
    PrimaryComponentTick.bCanEverTick = true;
    OSDisplayTexture = nullptr;
    FFIHandle = nullptr;
    OSInstance = nullptr;
}

void UPixelOSComponent::BeginPlay()
{
    Super::BeginPlay();
    LoadFFILibrary();
}

void UPixelOSComponent::EndPlay(const EEndPlayReason::Type EndPlayReason)
{
    UnloadFFILibrary();
    Super::EndPlay(EndPlayReason);
}

void UPixelOSComponent::TickComponent(float DeltaTime, ELevelTick TickType, FActorComponentTickFunction* ThisTickFunction)
{
    Super::TickComponent(DeltaTime, TickType, ThisTickFunction);

    if (OSInstance)
    {
        pixelos_step(OSInstance);
        UpdateTexture();
    }
}

void UPixelOSComponent::BootOS(FString OSName)
{
    if (pixelos_create)
    {
        OSConfig config = {1024, 768};
        OSInstance = pixelos_create(config);
        OSDisplayTexture = UTexture2D::CreateTransient(config.width, config.height, PF_R8G8B8A8);
    }
}

void UPixelOSComponent::LoadFFILibrary()
{
    FString ffiLibPath = FPaths::Combine(FPaths::ProjectPluginsDir(), TEXT("PixelOSPlugin"), TEXT("Source"), TEXT("ThirdParty"), TEXT("pixelos_ffi.so"));
    if (FPaths::FileExists(ffiLibPath))
    {
        FFIHandle = FPlatformProcess::GetDllHandle(*ffiLibPath);
        if (FFIHandle)
        {
            pixelos_create = (pixelos_create_t)FPlatformProcess::GetDllExport(FFIHandle, TEXT("pixelos_create"));
            pixelos_destroy = (pixelos_destroy_t)FPlatformProcess::GetDllExport(FFIHandle, TEXT("pixelos_destroy"));
            pixelos_step = (pixelos_step_t)FPlatformProcess::GetDllExport(FFIHandle, TEXT("pixelos_step"));
            pixelos_send_key = (pixelos_send_key_t)FPlatformProcess::GetDllExport(FFIHandle, TEXT("pixelos_send_key"));
            pixelos_get_framebuffer = (pixelos_get_framebuffer_t)FPlatformProcess::GetDllExport(FFIHandle, TEXT("pixelos_get_framebuffer"));
        }
    }
}

void UPixelOSComponent::UnloadFFILibrary()
{
    if (OSInstance)
    {
        pixelos_destroy(OSInstance);
        OSInstance = nullptr;
    }

    if (FFIHandle)
    {
        FPlatformProcess::FreeDllHandle(FFIHandle);
        FFIHandle = nullptr;
    }
}

void UPixelOSComponent::UpdateTexture()
{
    if (OSInstance && OSDisplayTexture)
    {
        const uint8* framebuffer = pixelos_get_framebuffer(OSInstance);
        if (framebuffer)
        {
            FTexture2DMipMap& Mip = OSDisplayTexture->PlatformData->Mips[0];
            void* TextureData = Mip.BulkData.Lock(LOCK_READ_WRITE);
            FMemory::Memcpy(TextureData, framebuffer, Mip.BulkData.GetBulkDataSize());
            Mip.BulkData.Unlock();
            OSDisplayTexture->UpdateResource();
        }
    }
}
