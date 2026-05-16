import json, re
import os

targetFiles = ["vscript","classnames","classnames_to_classes"]
filePath = os.path.dirname(os.path.abspath(__file__))
if os.path.isfile(filePath + "//entities_p2ce.json"):
    with open(filePath + "//entities_p2ce.json") as f:
        entitiesData = json.load(f)
else:
    print("\033[1;33m[WARNING] Could not find the \"entities_p2ce.json\" file, skipping to next file...\033[0m")
    targetFiles.remove("classnames")
    targetFiles.remove("classnames_to_classes")
    
if os.path.isfile(filePath + "//vscript_docs.server.json"):
    with open(filePath + "//vscript_docs.server.json") as f:
        vscriptData = json.load(f)
else:
    print("\033[1;33m[WARNING] Could not find the \"entities_p2ce.json\" file, skipping to next file...\033[0m")
    targetFiles.remove("vscript")

if len(targetFiles) == 0:
    print("\033[1;31m[ERROR] Could not find any of the needed files, please put the json files in the same folder of this python file.\033[0m")
    raise ValueError("Could not find needed files")


def parse_function(fdata,indent,insideclass):
    doc  = fdata.get('doc', '').strip()

    #Index all function data
    match = re.match(r'^(\S+)\s+(\S+)\((.*)\)$', fdata['signature'].strip())
    if match:
        ret,full_name,params_str = match.group(1), match.group(2), match.group(3)
    else:
        ret,full_name,params_str = 'void', fdata['signature'], ''
    
    #Parse params_str to separeted parameters
    if not params_str.strip():
        params = []
    else:
        parts = [p.strip() for p in params_str.split(',')]
        params = []
        for p in parts:
            tokens = p.split()
            if tokens[0] == "entity":
                tokens[0] = "CBaseEntity|null"
            if tokens[1] == "className":
                tokens[0] = "classname"
            if len(tokens) >= 2:
                params.append((tokens[0], tokens[-1]))
            elif tokens:
                params.append(('unknown', tokens[0]))

    if ret == "handle":
        ret = "CBaseEntity|null"

    if insideclass:
        name = full_name.split('::')[1] if '::' in full_name else full_name
        full_name = name

    #Build squirrel code
    code = []
    code.append(f'{indent}/**' \
    f'\n{indent} * {doc}' \
    f'\n{indent} *' \
    f'\n{indent} * @type {{function}}')
    for ptype, pname in params:
        code.append(f'{indent} * @param {{{ptype}}} {pname}')
    if ret != "void":
        code.append(f'{indent} * @returns {{{ret}}}')
    if ('deprecated' in doc.lower()):
        code.append(f'{indent} * @deprecated')
    code.append(f'{indent} */')
    code.append(f'{indent}function {full_name}({', '.join(name for _, name in params)});')
    code.append('')

    return '\n'.join(code)

def parse_class(cls):
    class_name = cls['class']
    extends    = cls.get('extends', '')
    #prevent extend to internal class
    if extends == "UTV_turretVoBlockedCooldownTimer":
        extends = ''
    methods    = cls.get('methods', [])

    class_code = []
    class_code.append(f'class {class_name}' + (f' extends {extends}' if extends else ''))
    class_code.append('{')

    for i, m in enumerate(methods):
        class_code.append(parse_function(m,'    ',True))

    class_code.append('}')
    class_code.append('')

    return '\n'.join(class_code)

print(f'[INFO] Parsing data from files...')
##--------------------
##VScript definitions parsing
##--------------------
if targetFiles.count("vscript") > 0:
    vscriptFile = []
    vscriptFile.append('/* ' \
    '\n * P2CE VScript definitions' \
    '\n * Generated using https://raw.githubusercontent.com/StrataSource/Wiki/refs/heads/main/dumps/vscript.json as reference' \
    '\n */')

    #append global functions
    vscriptFile.append('\n/*' \
    '\n * =======================' \
    '\n * GLOBAL FUNCTIONS' \
    '\n * =======================' \
    '\n */\n')

    for g in vscriptData['globals']:
        vscriptFile.append(parse_function(g,'',False))

    #append classes
    vscriptFile.append('\n/*' \
    '\n * =======================' \
    '\n * CLASSES' \
    '\n * =======================' \
    '\n */\n')
    for cls in vscriptData['classes']:
        vscriptFile.append(parse_class(cls))

    #append missing instances
    vscriptFile.append('/*' \
    '\n * =======================' \
    '\n * INSTANCES' \
    '\n * =======================' \
    '\n */\n')
    vscriptFile.append('\n/**' \
    '\n * Provides access to currently spawned entities.' \
    '\n * @type {CEntities}' \
    '\n * @const' \
    '\n*/'\
    '\nEntities <- CEntities()\n')
    vscriptFile.append('\n/**' \
    '\n * Contains the printed strings from the script_help command.' \
    '\n * @type {table}' \
    '\n*/' \
    '\nDocumentation <- {}')

    finalVScript = '\n'.join(vscriptFile)

##--------------------
##Classnames parsing (Classnames definitions can only be modified from source code, you need to compile the lsp server to update it)
##--------------------
if targetFiles.count("classnames") > 0:
    classnamesFile = []
    classnames_to_classFile = []
    ## Classnames from file parser
    for ent in entitiesData:
        classname = ent["classname"]
        classnamesFile.append(classname)
        if len(ent["bases"]) != 0:
            #print(ent['bases'][0])
            if ent["bases"][0] == "BaseEntityAnimating":
                classnames_to_classFile.append(classname + "$CBaseAnimating")
            elif ent["classname"] == "prop_portal":
                classnames_to_classFile.append(classname + "$CPropPortal")
            elif ent["classname"] == "linked_portal_door":
                classnames_to_classFile.append(classname + "$CLinkedPortalDoor")
            elif ent["classname"] == "prop_linked_portal_door":
                classnames_to_classFile.append(classname + "$CPropLinkedPortalDoor")
            elif ent["bases"][0] == "Weapon":
                classnames_to_classFile.append(classname + "$CBaseCombatWeapon")
            elif ent["classname"] == "point_viewcontrol":
                classnames_to_classFile.append(classname + "$CPointViewControl")
            elif ent["classname"] == "weapon_paintgun":
                classnames_to_classFile.append(classname + "$CWeaponPaintGun")
            elif ent["classname"] == "prop_physics_paintable": 
                classnames_to_classFile.append(classname + "$CPropPhysicsPaintable")
            elif ent["classname"] == "prop_weighted_cube": 
                classnames_to_classFile.append(classname + "$CPropWeightedCube")
            elif ent["classname"] == "env_entity_maker": 
                classnames_to_classFile.append(classname + "$CEnvEntityMaker")
            elif (len(ent["bases"]) > 1):
                if ent["bases"][1] == "BaseLight":
                    classnames_to_classFile.append(classname + "$CLight")
                else:
                    classnames_to_classFile.append(classname + "$CBaseEntity")
            else:
                classnames_to_classFile.append(classname + "$CBaseEntity")
    cnSorted = sorted(classnamesFile)
    cntcSorted = sorted(classnames_to_classFile)

    cnTxt = '\n'.join(cnSorted)
    cntcTxt = '\n'.join(cntcSorted)
for i,file in enumerate(targetFiles):
    print(f'[INFO] Generating file({i + 1}/{len(targetFiles)})...')
    if file == "vscript":
        with open(filePath + "/../vscript_lib/vscript.nut", 'w') as f:
            f.write(finalVScript)
            print(f'\033[1;32m[INFO] Generated \"vscript.nut\" file, located on \"./vscript_lib/vscript.nut\"!\033[0m')
    if file == "classnames":
        with open(filePath + ("/../crates/string_literals/data/classnames.txt"), 'w') as f:
            f.write(cnTxt)
            print(f'\033[1;32m[INFO] Generated \"classnames.txt\" file, located on \"./crates/string_literals/data/classnames.txt\"!\033[0m')
    if file == "classnames_to_classes":
        with open(filePath + ("/../crates/string_literals/data/classnames_to_classes.txt"), 'w') as f:
            f.write(cntcTxt)
            print(f'\033[1;32m[INFO] Generated \"classnames_to_classes.txt\" file, located on \"./crates/string_literals/data/classnames_to_classes.txt\"!\033[0m')