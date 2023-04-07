# Catalog Bundle Tool

A command-line interface tool and library to consult and edit Catalog.bundle files used by Unity games with the Addressables package.  
Very early version, only tested on Fire Emblem Engage. It should, however, be usable on other games. Help will not be provided to achieve this, unless you run into a bug in the tool (push requests welcome, though).

Use the ``-h`` argument for a list of supported commands.

## Example(s)
Here is an example JSON to add a model bundle to Fire Emblem Engage:
```json
{
  "bundles": [
    {
      "internal_id": "{UnityEngine.AddressableAssets.Addressables.RuntimePath}/Switch/fe_assets_unit/model/ubody/cor0af/c069/prefabs/ubody_cor0af_c069.bundle",
      "internal_path": "fe_assets_unit/model/ubody/cor0af/c069/prefabs/ubody_cor0af_c069.bundle"
    }
  ],
  "prefabs": [
    {
      "internal_id": "Assets/Share/Addressables/Unit/Model/uBody/Cor0AF/c069/Prefabs/uBody_Cor0AF_c069.prefab",
      "internal_path": "Unit/Model/uBody/Cor0AF/c069/Prefabs/uBody_Cor0AF_c069",
      "dependencies": [
        "{UnityEngine.AddressableAssets.Addressables.RuntimePath}/Switch/fe_assets_unit/model/ubody/cor0af/c069/prefabs/ubody_cor0af_c069.bundle"
      ]
    }
  ]
}

```

## Credits
Author and research: ``Raytwo``  
Special thanks and research: ``Moonling``  
Testers: ``Sierra``, ``DeathChaos25``  
Bundle support library: ``Thane98``
