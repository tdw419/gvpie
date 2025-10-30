using UnrealBuildTool;
using System.IO;

public class PixelOS : ModuleRules
{
    public PixelOS(ReadOnlyTargetRules Target) : base(Target)
    {
        PCHUsage = PCHUsageMode.UseExplicitOrSharedPCHs;

        PublicDependencyModuleNames.AddRange(new string[] { "Core", "CoreUObject", "Engine", "InputCore" });

        PrivateDependencyModuleNames.AddRange(new string[] { });

        string ffiLibPath = Path.Combine(PluginDirectory, "../../pixelos-ffi/target/release/libpixelos_ffi.so");
        PublicAdditionalLibraries.Add(ffiLibPath);
    }
}
