import unreal as ue

path = '/Game/_AssemblyStorm/TestMod/Gen/GenAsset'
print('asset exists? ' + str(ue.EditorAssetLibrary.does_asset_exist(asset_path=path)))
factory = ue.BlueprintFactory()
#factory.parent_class
parent_class = ue.EditorAssetLibrary.load_blueprint_class('/Game/_AssemblyStorm/TestMod/TestMod')
print(parent_class)
factory.set_editor_property("parent_class", parent_class)


ue.AssetToolsHelpers.get_asset_tools().create_asset(asset_name='GenAsset', package_path='/Game/_AssemblyStorm/TestMod/Gen/', asset_class=ue.Blueprint, factory=factory)

print('Hello i am your pipeline automator')

"""
anim_bp = ue.find_asset('/Game/Kaiju/Slicer/slicer_AnimBP.slicer_AnimBP')
if anim_bp:    
    ue.delete_asset(anim_bp.get_path_name())

# DoAttack custom event
node_do_attack = anim_bp.UberGraphPages[0].graph_add_node_custom_event('DoAttack', 0, -200)

# Boring custom event
node_boring = anim_bp.UberGraphPages[0].graph_add_node_custom_event('Boring', 0, -400)

# bool variables
ue.blueprint_add_member_variable(anim_bp, 'Attack', 'bool')
ue.blueprint_add_member_variable(anim_bp, 'Bored', 'bool')

# float variable
ue.blueprint_add_member_variable(anim_bp, 'Speed', 'float')
"""
