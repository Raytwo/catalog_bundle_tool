# Catalog Bundle Tool

A command-line interface tool and library to consult and edit Catalog.bundle files used by Unity games with the Addressables package.  
Very early version, only tested on Fire Emblem Engage. It should, however, be usable on other games. Help will not be provided to achieve this, unless you run into a bug in the tool (push requests welcome, though).

Use the ``-h`` argument for a list of supported commands.

## Example(s)
Here is an example TOML to add a model bundle to Fire Emblem Engage:
```toml
[[bundles]]
internal_id = "{UnityEngine.AddressableAssets.Addressables.RuntimePath}/Switch/fe_assets_unit/model/ubody/byl0am/c535/prefabs/ubody_byl0am_c535.bundle"
internal_path = "fe_assets_unit/model/ubody/byl0am/c535/prefabs/ubody_byl0am_c535_2727518c6675e8bc51a36f771de88f3f.bundle"

[[prefabs]]
internal_id = "Assets/Share/Addressables/Unit/Model/uBody/Byl0AM/c535/Prefabs/uBody_Byl0AM_c535.prefab"
internal_path = "Unit/Model/uBody/Byl0AM/c535/Prefabs/uBody_Byl0AM_c535"
dependencies = [
    "{UnityEngine.AddressableAssets.Addressables.RuntimePath}/Switch/fe_assets_unit/model/ubody/byl0am/c535/prefabs/ubody_byl0am_c535.bundle",
    "{UnityEngine.AddressableAssets.Addressables.RuntimePath}/Switch/fe_assets_unit/model/common/gradients_emblemw_metal.bundle",
    "{UnityEngine.AddressableAssets.Addressables.RuntimePath}/Switch/fe_assets_unit/model/common/gradients_emblemw_skin.bundle",
    "{UnityEngine.AddressableAssets.Addressables.RuntimePath}/Switch/fe_assets_customrp/shaders/chara/charastandard.shader.bundle",
    "{UnityEngine.AddressableAssets.Addressables.RuntimePath}/Switch/fe_assets_shaders/utils/fallbackerror.shader.bundle",
    "{UnityEngine.AddressableAssets.Addressables.RuntimePath}/Switch/fe_assets_unit/model/common/gradients_morph_metal.bundle",
    "{UnityEngine.AddressableAssets.Addressables.RuntimePath}/Switch/fe_assets_unit/model/common/gradients_emblemw_dress.bundle",
    "{UnityEngine.AddressableAssets.Addressables.RuntimePath}/Switch/fe_assets_unit/model/common/gradients_morph_dress.bundle",
]


```

## Credits
Author and research: ``Raytwo``  
Special thanks and research: ``Moonling``  
Testers: ``Sierra``, ``DeathChaos25``  
Bundle support library: ``Thane98``
