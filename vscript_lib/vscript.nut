/**
 * TF2 VScript Signatures
 * Generated from https://wiki.teamfortress.com/wiki/Team_Fortress_2/Scripting/Script_Functions
 * Only for reference, do not modify
 * @native
 */

// ============================================================
// CBaseEntity
// ============================================================

/**
 * Script handle class for entities. All entities have a script handle using this class,
 * sometimes as one of its subclasses.
 */
class CBaseEntity {
    /**
     * @param {string} key
     * @param {float} value
     * @returns {bool}
     * @deprecated Behaves the same as `KeyValueFromFloat`, use that instead.
     */
    function __KeyValueFromFloat(key, value);

    /**
     * @param {string} key
     * @param {integer} value
     * @returns {bool}
     * @deprecated Behaves the same as `KeyValueFromInt`, use that instead.
     */
    function __KeyValueFromInt(key, value);

    /**
     * @param {string} key
     * @param {string} value
     * @returns {bool}
     * @deprecated Behaves the same as `KeyValueFromString`, use that instead.
     */
    function __KeyValueFromString(key, value);

    /**
     * @param {string} key
     * @param {Vector} value
     * @returns {bool}
     * @deprecated Behaves the same as `KeyValueFromVector`, use that instead.
     */
    function __KeyValueFromVector(key, value);

    /**
     * Generate a synchronous I/O event. Unlike `EntFireByHandle`, this is processed immediately.
     * @param {input} input
     * @param {string|null} param
     * @param {CBaseEntity|null} activator
     * @param {CBaseEntity|null} caller
     * @returns {bool} `false` if input is a null/empty string, or if the input wasn't handled.
     */
    function AcceptInput(input, param, activator, caller);

    /**
     * Adds the supplied flags to the Entity Flags in the entity. (`m_iEFlags` datamap)
     *
     * **Note**: Adding `EFL_KILLME` will make the entity unkillable, even on round resets, until the flag is removed.
     * @param {integer} flags See [Constants.FPlayer](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FPlayer)
     */
    function AddEFlags(flags);

    /**
     * Adds the supplied flags to another separate player-related entity flags system in the entity. (`m_fFlags` datamap)
     * @param {integer} flags See [Constants.FPlayer](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FPlayer)
     */
    function AddFlag(flags);

    /**
     * Adds the supplied flags to the Solid Flags in the entity. (`m_Collision.m_usSolidFlags` datamap)
     * @param {integer} flags See [Constants.FSolid](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FSolid)
     */
    function AddSolidFlags(flags);

    /**
     * Apply a Velocity Impulse as a world space impulse vector.
     * Works for most physics-based objects including dropped weapons and even dropped Sandviches.
     * @param {Vector} impulse
     */
    function ApplyAbsVelocityImpulse(impulse);

    /**
     * Apply an Angular Velocity Impulse in entity local space.
     * The direction of the input vector is the rotation axis, and the length is the magnitude of the impulse.
     * @param {Vector} impulse
     */
    function ApplyLocalAngularVelocityImpulse(impulse);

    /**
     * Acts like the `BecomeRagdoll` input, with the required impulse value applied as a force on the ragdoll.
     * Does NOT spawn a prop_ragdoll or any other entity.
     *
     * **Warning**: These are a special group of ragdolls that never disappear by default.
     * @param {Vector} impulse
     * @returns {bool}
     */
    function BecomeRagdollOnClient(impulse);

    /**
     * Sets the player-related entity flags to 0 on an entity, clearing them.
     */
    function ClearFlags();

    /**
     * Sets Solid Flags to 0 on an entity, clearing them.
     */
    function ClearSolidFlags();

    /**
     * Adds an I/O connection that will call the named function when the specified output fires.
     * @param {string} output_name
     * @param {string} function_name
     */
    function ConnectOutput(output_name, function_name);

    /**
     * Removes the entity. Simply calls UTIL_Remove.
     */
    function Destroy();

    /**
     * Disable drawing and transmitting the entity to clients. (adds EF_NODRAW)
     */
    function DisableDraw();

    /**
     * Removes a connected script function from an I/O event.
     * @param {string} output_name
     * @param {string} function_name
     */
    function DisconnectOutput(output_name, function_name);

    /**
     * Alternative dispatch spawn, same as the one in CEntities, for convenience.
     *
     * **Note**: Calling this on players will cause them to respawn.
     */
    function DispatchSpawn();

    /**
     * Plays a sound from this entity. The sound must be precached first for it to play.
     *
     * **Warning**: Looping sounds will not stop on the entity when it's destroyed and will persist forever!
     * @param {string} sound_name
     */
    function EmitSound(sound_name);

    /**
     * Enable drawing and transmitting the entity to clients. (removes EF_NODRAW)
     */
    function EnableDraw();

    /**
     * Returns the entity index.
     * @returns {integer}
     */
    function entindex();

    /**
     * Returns the entity's eye angles. Acts like GetAbsAngles if the entity does not support it.
     * @returns {QAngle}
     */
    function EyeAngles();

    /**
     * Get vector to eye position - absolute coords. Acts like GetOrigin if the entity does not support it.
     * @returns {Vector}
     */
    function EyePosition();

    /**
     * Returns the most-recent entity parented to this one.
     * @returns {CBaseEntity|null}
     */
    function FirstMoveChild();

    /**
     * Get the entity's pitch, yaw, and roll as QAngles.
     * @returns {QAngle}
     */
    function GetAbsAngles();

    /**
     * Returns the current absolute velocity of the entity.
     * @returns {Vector}
     */
    function GetAbsVelocity();

    /**
     * Get the entity's pitch, yaw, and roll as Vector.
     * @returns {Vector}
     * @deprecated Use `GetAbsAngles` that returns a QAngle instead
     */
    function GetAngles();

    /**
     * Get the local angular velocity - returns a Vector of pitch, yaw, and roll.
     * @returns {Vector}
     */
    function GetAngularVelocity();

    /**
     * Returns any constant velocity currently being imparted onto the entity.
     * @returns {Vector}
     */
    function GetBaseVelocity();

    /**
     * Get a vector containing max bounds, centered on object.
     * @returns {Vector}
     */
    function GetBoundingMaxs();

    /**
     * Get a vector containing max bounds, centered on object, taking the object's orientation into account.
     * @returns {Vector}
     */
    function GetBoundingMaxsOriented();

    /**
     * Get a vector containing min bounds, centered on object.
     * @returns {Vector}
     */
    function GetBoundingMins();

    /**
     * Get a vector containing min bounds, centered on object, taking the object's orientation into account.
     * @returns {Vector}
     */
    function GetBoundingMinsOriented();

    /**
     * Gets center point of the entity in world coordinates.
     * @returns {Vector}
     */
    function GetCenter();

    /**
     * @returns {classname}
     */
    function GetClassname();

    /**
     * Gets the current collision group of the entity.
     * @returns {integer} See [Constants.ECollisionGroup](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ECollisionGroup)
     */
    function GetCollisionGroup();

    /**
     * Get the entity's engine flags.
     * @returns {integer} See [Constants.FEntityEFlags](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FEntityEFlags)
     */
    function GetEFlags();

    /**
     * Get the entity's flags.
     * @returns {integer} See [Constants.FPlayer](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FPlayer)
     */
    function GetFlags();

    /**
     * Get the entity as an EHANDLE.
     * @returns {instance}
     * @deprecated Leftover from earlier versions of VScript.
     */
    function GetEntityHandle();

    /**
     * @returns {integer}
     */
    function GetEntityIndex();

    /**
     * Get the forward vector of the entity.
     *
     * **Note**: If you intend to get a player's eye forward vector, use `EyeAngles().Forward()` instead.
     * @returns {Vector}
     */
    function GetForwardVector();

    /**
     * Get PLAYER friction, ignored for objects.
     * @returns {float}
     */
    function GetFriction();

    /**
     * @returns {float}
     */
    function GetGravity();

    /**
     * @returns {integer}
     */
    function GetHealth();

    /**
     * Get the right vector of the entity.
     * @returns {Vector}
     * @deprecated This is purely for compatibility, use `GetLeftVector` instead
     */
    function GetLeftVector();

    /**
     * @returns {QAngle}
     */
    function GetLocalAngles();

    /**
     * @returns {Vector}
     */
    function GetLocalOrigin();

    /**
     * Get Entity relative velocity.
     * @returns {Vector}
     */
    function GetLocalVelocity();

    /**
     * @returns {integer}
     */
    function GetMaxHealth();

    /**
     * Get a KeyValue class instance on this entity's model.
     * @returns {CScriptKeyValues}
     */
    function GetModelKeyValues();

    /**
     * Returns the name of the model.
     * @returns {string}
     */
    function GetModelName();

    /**
     * If in hierarchy, retrieves the entity's parent.
     * @returns {CBaseEntity|null}
     */
    function GetMoveParent();

    /**
     * @returns {integer}
     */
    function GetMoveType();

    /**
     * Get entity's targetname.
     * @returns {string}
     */
    function GetName();

    /**
     * This is GetAbsOrigin with a funny script name for some reason.
     * @returns {Vector}
     */
    function GetOrigin();

    /**
     * Gets this entity's owner.
     *
     * **Note**: This is a wrapper for `m_hOwnerEntity` netprop.
     * @returns {CBaseEntity|null}
     */
    function GetOwner();

    /**
     * @returns {Vector}
     */
    function GetPhysAngularVelocity();

    /**
     * @returns {Vector}
     */
    function GetPhysVelocity();

    /**
     * Get the entity name stripped of template unique decoration.
     * @returns {string}
     */
    function GetPreTemplateName();

    /**
     * Get the right vector of the entity.
     * @returns {Vector}
     */
    function GetRightVector();

    /**
     * If in hierarchy, walks up the hierarchy to find the root parent.
     * @returns {CBaseEntity|null}
     */
    function GetRootMoveParent();

    /**
     * Retrieve the unique identifier used to refer to the entity within the scripting system.
     * @returns {string}
     */
    function GetScriptId();

    /**
     * Retrieve the script-side data associated with an entity.
     * @returns {table|null}
     */
    function GetScriptScope();

    /**
     * Retrieve the name of the current script think func.
     * @returns {string}
     */
    function GetScriptThinkFunc();

    /**
     * @returns {integer} See [Constants.ESolidType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ESolidType)
     */
    function GetSolid();

    /**
     * Returns float duration of the sound. Actor model name is optional and can be left empty.
     *
     * **Warning**: Does not work on dedicated servers.
     * @param {string} sound_name
     * @param {string|null} actor_model_name
     * @returns {float}
     */
    function GetSoundDuration(sound_name, actor_model_name);

    /**
     * @returns {integer} See [Constants.ETFTeam](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFTeam)
     */
    function GetTeam();

    /**
     * Get the up vector of the entity.
     * @returns {Vector}
     */
    function GetUpVector();

    /**
     * @returns {Vector}
     * @deprecated Use `GetAbsVelocity` instead
     */
    function GetVelocity();

    /**
     * This function tells you how much of the entity is underwater.
     * @returns {integer} 0=not underwater, 1=feet, 2=waist, 3=head.
     */
    function GetWaterLevel();

    /**
     * Returns the type of water the entity is currently submerged in.
     * @returns {integer} 32=water, 16=slime.
     */
    function GetWaterType();

    /**
     * Am I alive?
     * @returns {bool}
     */
    function IsAlive();

    /**
     * @param {integer} flag See [Constants.FEntityEFlags](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FEntityEFlags)
     * @returns {bool}
     */
    function IsEFlagSet(flag);

    /**
     * Checks whether the entity is a player or not.
     * @returns {bool}
     */
    function IsPlayer();

    /**
     * @returns {bool}
     */
    function IsSolid();

    /**
     * @param {integer} flag See [Constants.FSolid](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FSolid)
     * @returns {bool}
     */
    function IsSolidFlagSet(flag);

    /**
     * Checks whether the entity still exists.
     * Useful when storing entity handles and needing to check if the entity was not deleted.
     * @returns {bool}
     */
    function IsValid();

    /**
     * Executes KeyValue with a float.
     *
     * **Warning**: Does not update the internal network state of the entity.
     * @param {string} key
     * @param {float} value
     * @returns {bool}
     */
    function KeyValueFromFloat(key, value);

    /**
     * Executes KeyValue with an int.
     *
     * **Warning**: Does not update the internal network state of the entity.
     * @param {string} key
     * @param {integer} value
     * @returns {bool}
     */
    function KeyValueFromInt(key, value);

    /**
     * Executes KeyValue with a string.
     *
     * **Warning**: Does not update the internal network state of the entity.
     * @param {string} key
     * @param {string} value
     * @returns {bool}
     */
    function KeyValueFromString(key, value);

    /**
     * Executes KeyValue with a vector.
     *
     * **Warning**: Does not update the internal network state of the entity.
     * @param {string} key
     * @param {Vector} value
     * @returns {bool}
     */
    function KeyValueFromVector(key, value);

    /**
     * Removes the entity. Equivalent of firing the Kill I/O input, but instantaneous.
     *
     * **Warning**: This clears the owner entity before removing.
     */
    function Kill();

    /**
     * Returns the entity's local eye angles.
     * @returns {QAngle}
     */
    function LocalEyeAngles();

    /**
     * Returns the next entity parented with the entity.
     * @returns {CBaseEntity|null}
     */
    function NextMovePeer();

    /**
     * Precache a model (.mdl) or sprite (.vmt). The extension must be specified.
     * @param {string} model_name
     */
    function PrecacheModel(model_name);

    /**
     * Precache a soundscript or raw WAV/MP3 sound. Same as PrecacheSoundScript.
     * @param {string} sound_script
     */
    function PrecacheScriptSound(sound_script);

    /**
     * Precache a soundscript or raw WAV/MP3 sound. Same as PrecacheScriptSound.
     * @param {string} sound_script
     */
    function PrecacheSoundScript(sound_script);

    /**
     * @param {integer} flags See [Constants.FEntityEFlags](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FEntityEFlags)
     */
    function RemoveEFlags(flags);

    /**
     * @param {integer} flags See [Constants.FPlayer](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FPlayer)
     */
    function RemoveFlag(flags);

    /**
     * @param {integer} flags See [Constants.FSolid](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FSolid)
     */
    function RemoveSolidFlags(flags);

    /**
     * Set entity pitch, yaw, roll as QAngles. Does not work on players, use SnapEyeAngles instead.
     * @param {QAngle} angles
     */
    function SetAbsAngles(angles);

    /**
     * Sets the current absolute velocity of the entity.
     * Does nothing on VPhysics objects, use SetPhysVelocity instead.
     * @param {Vector} velocity
     */
    function SetAbsVelocity(velocity);

    /**
     * Sets the absolute origin of the entity.
     * @param {Vector} origin
     */
    function SetAbsOrigin(origin);

    /**
     * Sets entity angles.
     * @param {float} pitch
     * @param {float} yaw
     * @param {float} roll
     * @deprecated Use `SetAbsAngles` instead
     */
    function SetAngles(pitch, yaw, roll);

    /**
     * Set the local angular velocity.
     * @param {float} pitch
     * @param {float} yaw
     * @param {float} roll
     */
    function SetAngularVelocity(pitch, yaw, roll);

    /**
     * Set the current collision group of the entity.
     * @param {integer} group See [Constants.ECollisionGroup](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ECollisionGroup)
     */
    function SetCollisionGroup(group);

    /**
     * Enables drawing if you pass true, disables drawing if you pass false.
     * @param {bool} toggle
     */
    function SetDrawEnabled(toggle);

    /**
     * @param {integer} flags See [Constants.FEntityEFlags](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FEntityEFlags)
     */
    function SetEFlags(flags);

    /**
     * Set the orientation of the entity to have this forward vector.
     * @param {Vector} forward
     */
    function SetForwardVector(forward);

    /**
     * @param {float} friction
     */
    function SetFriction(friction);

    /**
     * Sets a multiplier for gravity. 1 is default gravity.
     *
     * **Note**: 0 gravity will not work, use 0.000001 as a workaround.
     * @param {float} gravity
     */
    function SetGravity(gravity);

    /**
     * @param {integer} health
     */
    function SetHealth(health);

    /**
     * @param {QAngle} angles
     */
    function SetLocalAngles(angles);

    /**
     * @param {Vector} origin
     */
    function SetLocalOrigin(origin);

    /**
     * Sets the maximum health this entity can have. Does not update the current health.
     *
     * **Note**: Does nothing on players.
     * @param {integer} health
     */
    function SetMaxHealth(health);

    /**
     * Set a model for this entity.
     *
     * **Warning**: Make sure the model was already precached before using this function or the game will crash!
     * @param {string} model_name
     */
    function SetModel(model_name);

    /**
     * @param {integer} movetype See [Constants.EMoveType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#EMoveType)
     * @param {integer} movecollide See [Constants.EMoveCollide](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#EMoveCollide)
     */
    function SetMoveType(movetype, movecollide);

    /**
     * @param {Vector} origin
     * @deprecated Use `SetAbsOrigin` instead
     */
    function SetOrigin(origin);

    /**
     * Sets this entity's owner.
     *
     * **Note**: This is a wrapper for `m_hOwnerEntity` netprop.
     * @param {CBaseEntity|null} entity
     */
    function SetOwner(entity);

    /**
     * @param {Vector} angular_velocity
     */
    function SetPhysAngularVelocity(angular_velocity);

    /**
     * @param {Vector} velocity
     */
    function SetPhysVelocity(velocity);

    /**
     * Sets the bounding box's scale for this entity.
     *
     * **Warning**: If any component of mins/maxs is backwards, the engine will exit with a fatal error.
     * @param {Vector} mins
     * @param {Vector} maxs
     */
    function SetSize(mins, maxs);

    /**
     * @param {integer} solid See [Constants.ESolidType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ESolidType)
     */
    function SetSolid(solid);

    /**
     * @param {integer} flags See [Constants.FSolid](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FSolid)
     */
    function SetSolidFlags(flags);

    /**
     * Sets entity team.
     *
     * **Note**: Use `ForceChangeTeam` on players instead.
     * @param {integer} team See [Constants.ETFTeam](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFTeam)
     */
    function SetTeam(team);

    /**
     * @param {Vector} velocity
     * @deprecated Use `SetAbsVelocity` instead
     */
    function SetVelocity(velocity);

    /**
     * Sets how much of the entity is underwater. 0=not underwater, 1=feet, 2=waist, 3=head.
     * @param {integer} water_level See [Constants.WATERLEVEL](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#WATERLEVEL)
     */
    function SetWaterLevel(water_level);

    /**
     * Set the type of water the entity is currently submerged in. 32=water, 16=slime.
     * @param {integer} water_type
     */
    function SetWaterType(water_type);

    /**
     * Stops a sound on this entity.
     * @param {string} sound_name
     */
    function StopSound(sound_name);

    /**
     * Deals damage to the entity.
     * @param {float} damage
     * @param {integer} damage_type See [Constants.FDmgType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FDmgType)
     * @param {CBaseEntity} attacker
     */
    function TakeDamage(damage, damage_type, attacker);

    /**
     * Extended version of TakeDamage.
     *
     * **Note**: If damage_force is Vector(0,0,0), the game will automatically calculate it.
     * @param {CBaseEntity|null} inflictor
     * @param {CBaseEntity|null} attacker
     * @param {CBaseEntity|null} weapon
     * @param {Vector} damage_force
     * @param {Vector} damage_position
     * @param {float} damage
     * @param {integer} damage_type See [Constants.FDmgType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FDmgType)
     */
    function TakeDamageEx(inflictor, attacker, weapon, damage_force, damage_position, damage, damage_type);

    /**
     * Extended version of TakeDamageEx that can apply a custom damage type.
     * @param {CBaseEntity|null} inflictor
     * @param {CBaseEntity|null} attacker
     * @param {CBaseEntity|null} weapon
     * @param {Vector} damage_force
     * @param {Vector} damage_position
     * @param {float} damage
     * @param {integer} damage_type See [Constants.FDmgType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FDmgType)
     * @param {integer} custom_damage_type See [Constants.ETFDmgCustom](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFDmgCustom)
     */
    function TakeDamageCustom(inflictor, attacker, weapon, damage_force, damage_position, damage, damage_type, custom_damage_type);

    /**
     * Teleports this entity. Set bools to false for properties you want unchanged.
     * @param {bool} use_origin
     * @param {Vector} origin
     * @param {bool} use_angles
     * @param {QAngle} angles
     * @param {bool} use_velocity
     * @param {Vector} velocity
     */
    function Teleport(use_origin, origin, use_angles, angles, use_velocity, velocity);

    /**
     * Clear the current script scope for this entity.
     */
    function TerminateScriptScope();

    /**
     * @param {integer} flags See [Constants.FPlayer](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FPlayer)
     */
    function ToggleFlag(flags);

    /**
     * Create a script scope for an entity if it doesn't already exist.
     * @returns {bool}
     */
    function ValidateScriptScope();
}

// ============================================================
// CBaseAnimating extends CBaseEntity
// ============================================================

/**
 * Script handle class for animatable entities, such as props.
 */
class CBaseAnimating extends CBaseEntity {
    /**
     * Dispatch animation events to a CBaseAnimating entity.
     * @param {CBaseAnimating} entity
     */
    function DispatchAnimEvents(entity);

    /**
     * Find a bodygroup ID by name. Returns -1 if the bodygroup does not exist.
     * @param {string} name
     * @returns {integer}
     */
    function FindBodygroupByName(name);

    /**
     * Get an attachment's angles as a QAngle, by ID.
     * @param {integer} id
     * @returns {QAngle}
     */
    function GetAttachmentAngles(id);

    /**
     * Get an attachment's parent bone index by ID.
     * @param {integer} id
     * @returns {integer}
     */
    function GetAttachmentBone(id);

    /**
     * Get an attachment's origin as a Vector, by ID.
     * @param {integer} id
     * @returns {Vector}
     */
    function GetAttachmentOrigin(id);

    /**
     * Get the bodygroup value by bodygroup ID.
     * @param {integer} id
     * @returns {integer}
     */
    function GetBodygroup(id);

    /**
     * Get the bodygroup's name by ID.
     * @param {integer} id
     * @returns {string}
     */
    function GetBodygroupName(id);

    /**
     * Get the bodygroup's name by group and part.
     * @param {integer} group
     * @param {integer} part
     * @returns {string}
     */
    function GetBodygroupPartName(group, part);

    /**
     * Get the bone's angles as a QAngle, by ID.
     *
     * **Warning**: Bone transforms are cached; setting new sequences may cause stale bone data.
     * @param {integer} id
     * @returns {QAngle}
     */
    function GetBoneAngles(id);

    /**
     * Get the bone's origin Vector by ID.
     * **Warning**: See GetBoneAngles warning.
     * @param {integer} id
     * @returns {Vector}
     */
    function GetBoneOrigin(id);

    /**
     * Gets the model's current animation cycle rate. Ranges from 0.0 to 1.0.
     * @returns {float}
     */
    function GetCycle();

    /**
     * Get the model's scale.
     * @returns {float}
     */
    function GetModelScale();

    /**
     * Get the current animation's playback rate.
     * @returns {float}
     */
    function GetPlaybackRate();

    /**
     * Get the current-playing sequence's ID.
     * @returns {integer}
     */
    function GetSequence();

    /**
     * Get the activity name for a sequence by sequence ID.
     * @param {integer} id
     * @returns {string}
     */
    function GetSequenceActivityName(id);

    /**
     * Get a sequence duration in seconds by sequence ID.
     * @param {integer} id
     * @returns {float}
     */
    function GetSequenceDuration(id);

    /**
     * Get a sequence name by sequence ID.
     * @param {integer} id
     * @returns {string}
     */
    function GetSequenceName(id);

    /**
     * Gets the current skin index.
     * @returns {integer}
     */
    function GetSkin();

    /**
     * Ask whether the main sequence is done playing.
     * @returns {bool}
     */
    function IsSequenceFinished();

    /**
     * Get the named activity index. Returns -1 if the activity does not exist.
     * @param {string} activity
     * @returns {integer}
     */
    function LookupActivity(activity);

    /**
     * Get the named attachment index. Returns 0 if the attachment does not exist.
     * @param {string} name
     * @returns {integer}
     */
    function LookupAttachment(name);

    /**
     * Get the named bone index. Returns -1 if the bone does not exist.
     * @param {string} bone
     * @returns {integer}
     */
    function LookupBone(bone);

    /**
     * Gets the pose parameter's index. Returns -1 if the pose parameter does not exist.
     * @param {string} name
     * @returns {integer}
     */
    function LookupPoseParameter(name);

    /**
     * Looks up a sequence by names of sequences or activities. Returns -1 if not found.
     * @param {string} name
     * @returns {integer}
     */
    function LookupSequence(name);

    /**
     * Reset a sequence by sequence ID. If the ID is different, switch to the new sequence.
     * @param {integer} id
     */
    function ResetSequence(id);

    /**
     * Set the bodygroup by ID.
     * @param {integer} id
     * @param {integer} value
     */
    function SetBodygroup(id, value);

    /**
     * Sets the model's current animation cycle from 0 to 1.
     *
     * **Note**: Only works if `m_bClientSideAnimation` is set to false.
     * @param {float} cycle
     */
    function SetCycle(cycle);

    /**
     * Set a model for this entity. Automatically precaches and maintains sequence/cycle if possible.
     * @param {string} model_name
     */
    function SetModelSimple(model_name);

    /**
     * Changes a model's scale over time. Set change_duration to 0.0 to change instantly.
     * @param {float} scale
     * @param {float} change_duration
     */
    function SetModelScale(scale, change_duration);

    /**
     * Set the current animation's playback rate.
     * @param {float} rate
     */
    function SetPlaybackRate(rate);

    /**
     * Sets a pose parameter value. Returns the effective value after clamping or looping.
     * @param {integer} id
     * @param {float} value
     * @returns {float}
     */
    function SetPoseParameter(id, value);

    /**
     * Plays a sequence by sequence ID.
     *
     * **Warning**: Can cause animation stutters. Consider using ResetSequence instead.
     * @param {integer} id
     */
    function SetSequence(id);

    /**
     * Sets the model's skin.
     * @param {integer} index
     */
    function SetSkin(index);

    /**
     * Stop the current animation (same as SetPlaybackRate 0.0).
     */
    function StopAnimation();

    /**
     * Advance animation frame to some time in the future with an automatically calculated interval.
     */
    function StudioFrameAdvance();

    /**
     * Advance animation frame to some time in the future with a manual interval.
     * @param {float} dt
     */
    function StudioFrameAdvanceManual(dt);
}

// ============================================================
// CBaseCombatWeapon extends CBaseAnimating
// ============================================================

/**
 * Script handle class for any weapon entities that can be part of a player's inventory.
 */
class CBaseCombatWeapon extends CBaseAnimating {
    /**
     * Can this weapon be selected.
     * @returns {bool}
     */
    function CanBeSelected();

    /**
     * Current ammo in clip1. Returns -1 if clip1 is not present.
     * @returns {integer}
     */
    function Clip1();

    /**
     * Current ammo in clip2. Returns -1 if clip2 is not present.
     * @returns {integer}
     */
    function Clip2();

    /**
     * Default size of clip1. Returns -1 if clip1 is not present.
     * @returns {integer}
     */
    function GetDefaultClip1();

    /**
     * Default size of clip2. Returns -1 if clip2 is not present.
     * @returns {integer}
     */
    function GetDefaultClip2();

    /**
     * Max size of clip1. Returns -1 if clip1 is not present.
     * @returns {integer}
     */
    function GetMaxClip1();

    /**
     * Max size of clip2. Returns -1 if clip2 is not present.
     * @returns {integer}
     */
    function GetMaxClip2();

    /**
     * Gets the weapon's internal name (not the targetname!)
     *
     * **Warning**: Conflicts with CBaseEntity's GetName. Use CBaseEntity.GetName.call(weapon) for targetname.
     * @returns {string}
     */
    function GetName();

    /**
     * Gets the weapon's current position.
     * @returns {integer}
     */
    function GetPosition();

    /**
     * Current primary ammo count.
     * @returns {integer}
     */
    function GetPrimaryAmmoCount();

    /**
     * Returns the primary ammo type.
     * @returns {integer}
     */
    function GetPrimaryAmmoType();

    /**
     * Gets the weapon's print name.
     * @returns {string}
     */
    function GetPrintName();

    /**
     * Current secondary ammo count.
     * @returns {integer}
     */
    function GetSecondaryAmmoCount();

    /**
     * Returns the secondary ammo type.
     * @returns {integer}
     */
    function GetSecondaryAmmoType();

    /**
     * Gets the weapon's current slot.
     * @returns {integer}
     */
    function GetSlot();

    /**
     * Get the weapon subtype.
     * @returns {integer}
     */
    function GetSubType();

    /**
     * Get the weapon flags.
     * @returns {integer}
     */
    function GetWeaponFlags();

    /**
     * Get the weapon weighting/importance.
     * @returns {integer}
     */
    function GetWeight();

    /**
     * Do we have any ammo?
     * @returns {bool}
     */
    function HasAnyAmmo();

    /**
     * Do we have any primary ammo?
     * @returns {bool}
     */
    function HasPrimaryAmmo();

    /**
     * Do we have any secondary ammo?
     * @returns {bool}
     */
    function HasSecondaryAmmo();

    /**
     * Are we allowed to switch to this weapon?
     * @returns {bool}
     */
    function IsAllowedToSwitch();

    /**
     * Returns whether this is a melee weapon.
     * @returns {bool}
     */
    function IsMeleeWeapon();

    /**
     * Force a primary attack.
     *
     * **Warning**: Hitscan and melee weapons require lag compensation information to be present.
     */
    function PrimaryAttack();

    /**
     * Force a secondary attack.
     *
     * **Warning**: Hitscan and melee weapons require lag compensation information to be present.
     */
    function SecondaryAttack();

    /**
     * Set current ammo in clip1.
     * @param {integer} amount
     */
    function SetClip1(amount);

    /**
     * Set current ammo in clip2.
     * @param {integer} amount
     */
    function SetClip2(amount);

    /**
     * Sets a custom view model for this weapon by model name.
     * @param {string} model_name
     */
    function SetCustomViewModel(model_name);

    /**
     * Sets a custom view model for this weapon by modelindex.
     * @param {integer} model_index
     */
    function SetCustomViewModelModelIndex(model_index);

    /**
     * Set the weapon subtype.
     * @param {integer} subtype
     */
    function SetSubType(subtype);

    /**
     * Do we use clips for ammo 1?
     * @returns {bool}
     */
    function UsesClipsForAmmo1();

    /**
     * Do we use clips for ammo 2?
     * @returns {bool}
     */
    function UsesClipsForAmmo2();

    /**
     * Do we use primary ammo?
     * @returns {bool}
     */
    function UsesPrimaryAmmo();

    /**
     * Do we use secondary ammo?
     * @returns {bool}
     */
    function UsesSecondaryAmmo();

    /**
     * Is this weapon visible in weapon selection?
     * @returns {bool}
     */
    function VisibleInWeaponSelection();
}

/**
 * This is just multiple inheritance of CBaseCombatWeapon and CEconEntity
 * with no additional methods added. Here it inherits CBaseCombatWeapon
 * and copies CEconEntity methods to achieve the same result. (Why C++
 * developers are spreading their broken OOP curse on everyone else?)
 * @extends {CBaseAnimating|CEconEntity}
 */
class CTFWeaponBase extends CBaseCombatWeapon {
    /**
     * Add an attribute to the entity. Set duration to 0 or lower for infinite duration.
     *
     * **Note**: For players use `AddCustomAttribute` instead.
     * @param {attribute} name
     * @param {float} value
     * @param {float} duration
     */
    function AddAttribute(name, value, duration);

    /**
     * Get an attribute float from the entity. Returns `default_value` if not found.
     * @param {attribute} name
     * @param {float} default_value
     * @returns {float}
     */
    function GetAttribute(name, default_value);

    /**
     * Remove an attribute from the entity.
     *
     * **Note**: Static attributes cannot be removed with this method.
     * @param {attribute} name
     */
    function RemoveAttribute(name);

    /**
     * Relinks attributes to provisioners.
     */
    function ReapplyProvision();
}

// ============================================================
// CBaseFlex extends CBaseAnimating
// ============================================================

/**
 * Animated characters who have vertex flex capability (e.g., facial expressions).
 */
class CBaseFlex extends CBaseAnimating {
    /**
     * Play the specified .vcd file, causing the related characters to speak and subtitles to play.
     * @param {string} scene_file
     * @param {float} delay
     * @returns {float}
     */
    function PlayScene(scene_file, delay);
}

// ============================================================
// CBaseCombatCharacter extends CBaseFlex
// ============================================================

/**
 * Combat entities with similar movement capabilities to a player.
 */
class CBaseCombatCharacter extends CBaseFlex {
    /**
     * Return the last nav area occupied, null if unknown.
     * @returns {CTFNavArea|null}
     */
    function GetLastKnownArea();
}

// ============================================================
// CBasePlayer extends CBaseCombatCharacter
// ============================================================

/**
 * Script handle class for player entities.
 */
class CBasePlayer extends CBaseCombatCharacter {
    /**
     * Whether the player is being forced by SetForceLocalDraw to be drawn.
     * @returns {bool}
     */
    function GetForceLocalDraw();

    /**
     * Get a vector containing max bounds of the player in local space.
     * @returns {Vector}
     */
    function GetPlayerMaxs();

    /**
     * Get a vector containing min bounds of the player in local space.
     * @returns {Vector}
     */
    function GetPlayerMins();

    /**
     * Gets the current overlay material set by SetScriptOverlayMaterial.
     * @returns {string}
     */
    function GetScriptOverlayMaterial();

    /**
     * Returns true if the player is in noclip mode.
     * @returns {bool}
     */
    function IsNoclipping();

    /**
     * Forces the player to be drawn as if they were in thirdperson.
     * @param {bool} toggle
     */
    function SetForceLocalDraw(toggle);

    /**
     * Sets the overlay material that can't be overridden by other overlays.
     * @param {string|null} material
     */
    function SetScriptOverlayMaterial(material);

    /**
     * Snap the player's eye angles to this.
     * @param {QAngle} angles
     */
    function SnapEyeAngles(angles);

    /**
     * Ow! Punches the player's view.
     * @param {QAngle} angle_offset
     */
    function ViewPunch(angle_offset);

    /**
     * Resets the player's view punch if the offset stays below the given tolerance.
     * @param {float} tolerance
     */
    function ViewPunchReset(tolerance);
}

// ============================================================
// CEconEntity extends CBaseAnimating
// ============================================================

/**
 * Script handle class for economic equippables (hats and weapons).
 */
class CEconEntity extends CBaseAnimating {
    /**
     * Add an attribute to the entity. Set duration to 0 or lower for infinite duration.
     *
     * **Note**: For players use AddCustomAttribute instead.
     * @param {attribute} name
     * @param {float} value
     * @param {float} duration
     */
    function AddAttribute(name, value, duration);

    /**
     * Get an attribute float from the entity. Returns default_value if not found.
     * @param {attribute} name
     * @param {float} default_value
     * @returns {float}
     */
    function GetAttribute(name, default_value);

    /**
     * Remove an attribute from the entity.
     *
     * **Note**: Static attributes cannot be removed with this method.
     * @param {attribute} name
     */
    function RemoveAttribute(name);

    /**
     * Relinks attributes to provisioners.
     */
    function ReapplyProvision();
}

// ============================================================
// CTFPlayer extends CBasePlayer (and CEconEntity indirectly)
// ============================================================

/**
 * Script handle class for player entities of Team Fortress 2.
 */
class CTFPlayer extends CBasePlayer {
    /**
     * @param {integer} cond See [Constants.ETFCond](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFCond)
     */
    function AddCond(cond);

    /**
     * @param {integer} cond See [Constants.ETFCond](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFCond)
     * @param {float} duration
     * @param {CBaseEntity|null} provider
     */
    function AddCondEx(cond, duration, provider);

    /**
     * Give the player some cash for MvM. New value is bounded between 0-30000.
     * @param {integer} amount
     */
    function AddCurrency(amount);

    /**
     * Add a custom attribute to the player. Set duration to 0 or lower for infinite.
     *
     * **Note**: Does not work when applied in the player_spawn event.
     * @param {attribute} name
     * @param {float} value
     * @param {float} duration
     */
    function AddCustomAttribute(name, value, duration);

    /**
     * Hides a HUD element(s).
     * @param {integer} flags See [Constants.FHideHUD](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FHideHUD)
     */
    function AddHudHideFlags(flags);

    /**
     * Apply a view punch along the pitch angle. Returns true if the punch was applied.
     * @param {float} impulse
     * @returns {bool}
     */
    function ApplyPunchImpulseX(impulse);

    /**
     * Make a player bleed for a set duration of time.
     * @param {float} duration
     */
    function BleedPlayer(duration);

    /**
     * Make a player bleed with specific damage per tick and custom damage type.
     * @param {float} duration
     * @param {integer} damage
     * @param {bool} endless
     * @param {integer} custom_damage_type See [Constants.ETFDmgCustom](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFDmgCustom)
     */
    function BleedPlayerEx(duration, damage, endless, custom_damage_type);

    /**
     * Cancels any taunt in progress.
     */
    function CancelTaunt();

    /**
     * Can the player air dash/double jump?
     * @returns {bool}
     */
    function CanAirDash();

    /**
     * @returns {bool}
     */
    function CanBeDebuffed();

    /**
     * @returns {bool}
     */
    function CanBreatheUnderwater();

    /**
     * Can the player duck?
     * @returns {bool}
     */
    function CanDuck();

    /**
     * Can the player get wet by jarate/milk?
     * @returns {bool}
     */
    function CanGetWet();

    /**
     * Can the player jump?
     * @returns {bool}
     */
    function CanJump();

    /**
     * Can the player move?
     * @returns {bool}
     */
    function CanPlayerMove();

    /**
     */
    function ClearCustomModelRotation();

    /**
     */
    function ClearSpells();

    /**
     * Stops active taunt from damaging or cancels Rock-Paper-Scissors result.
     */
    function ClearTauntAttack();

    /**
     * Performs taunts attacks if available.
     */
    function DoTauntAttack();

    /**
     * Force player to drop the flag (intelligence).
     * @param {bool} silent
     */
    function DropFlag(silent);

    /**
     * Force player to drop the rune.
     * @param {bool} apply_force
     * @param {integer} team See [Constants.ETFTeam](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFTeam)
     */
    function DropRune(apply_force, team);

    /**
     * Stops a looping taunt (obeys minimum time rules).
     */
    function EndLongTaunt();

    /**
     * Equips a wearable on the viewmodel.
     * @param {CBaseEntity} entity
     */
    function EquipWearableViewModel(entity);

    /**
     */
    function ExtinguishPlayerBurning();

    /**
     * Makes e.g. a heavy go AAAAAAAAAaAaa like they are firing their minigun.
     */
    function FiringTalk();

    /**
     * Force player to change their team.
     * @param {integer} team See [Constants.ETFTeam](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFTeam)
     * @param {bool} full_team_switch
     */
    function ForceChangeTeam(team, full_team_switch);

    /**
     * Force regenerates and respawns the player.
     */
    function ForceRegenerateAndRespawn();

    /**
     * Force respawns the player.
     */
    function ForceRespawn();

    /**
     * Get the player's current weapon.
     * @returns {CTFWeaponBase}
     */
    function GetActiveWeapon();

    /**
     * @returns {integer}
     */
    function GetBackstabs();

    /**
     * @returns {integer}
     */
    function GetBonusPoints();

    /**
     * @returns {integer}
     */
    function GetBotType();

    /**
     * @returns {integer}
     */
    function GetBuildingsDestroyed();

    /**
     * @returns {integer}
     */
    function GetCaptures();

    /**
     * Gets the eye height of the player.
     * @returns {Vector}
     */
    function GetClassEyeHeight();

    /**
     * Returns duration of the condition. Returns 0 if not applied. Returns -1 if infinite.
     * @param {integer} cond See [Constants.ETFCond](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFCond)
     * @returns {float}
     */
    function GetCondDuration(cond);

    /**
     * Get an attribute float from the player. Returns default_value if not found.
     * @param {attribute} name
     * @param {float} default_value
     * @returns {float}
     */
    function GetCustomAttribute(name, default_value);

    /**
     * Get player's cash for MvM.
     * @returns {integer}
     */
    function GetCurrency();

    /**
     * @returns {float}
     */
    function GetCurrentTauntMoveSpeed();

    /**
     * @returns {integer}
     */
    function GetDefenses();

    /**
     * @returns {integer}
     */
    function GetDisguiseAmmoCount();

    /**
     * @returns {CTFPlayer|null}
     */
    function GetDisguiseTarget();

    /**
     * @returns {integer}
     */
    function GetDisguiseTeam();

    /**
     * @returns {integer}
     */
    function GetDominations();

    /**
     * What entity is the player grappling?
     * @returns {CBaseEntity|null}
     */
    function GetGrapplingHookTarget();

    /**
     * @returns {integer}
     */
    function GetHeadshots();

    /**
     * @returns {integer}
     */
    function GetHealPoints();

    /**
     * Who is the medic healing?
     * @returns {CBaseEntity|null}
     */
    function GetHealTarget();

    /**
     * Gets current hidden HUD elements.
     * @returns {integer} See [Constants.FHideHUD](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FHideHUD)
     */
    function GetHudHideFlags();

    /**
     * @returns {integer}
     */
    function GetInvulns();

    /**
     * @returns {integer}
     */
    function GetKillAssists();

    /**
     * @returns {CTFWeaponBase|null}
     */
    function GetLastWeapon();

    /**
     * Get next change class time.
     * @returns {float}
     */
    function GetNextChangeClassTime();

    /**
     * Get next change team time.
     * @returns {float}
     */
    function GetNextChangeTeamTime();

    /**
     * Get next health regen time.
     * @returns {float}
     */
    function GetNextRegenTime();

    /**
     * @returns {integer}
     */
    function GetPlayerClass();

    /**
     * @returns {float}
     */
    function GetRageMeter();

    /**
     * @returns {integer}
     */
    function GetResupplyPoints();

    /**
     * @returns {integer}
     */
    function GetRevenge();

    /**
     * @returns {float}
     */
    function GetScoutHypeMeter();

    /**
     * @returns {float}
     */
    function GetSpyCloakMeter();

    /**
     * @returns {integer}
     */
    function GetTeleports();

    /**
     * Timestamp until a taunt attack lasts. 0 if unavailable.
     * @returns {float}
     */
    function GetTauntAttackTime();

    /**
     * Timestamp until taunt is stopped.
     * @returns {float}
     */
    function GetTauntRemoveTime();

    /**
     * Timestamp when kart was reversed. FLT_MAX if yet to be done.
     * @returns {float}
     */
    function GetVehicleReverseTime();

    /**
     * When did the player last call medic. 99999.9 if yet to be done.
     * @returns {float}
     */
    function GetTimeSinceCalledForMedic();

    /**
     * @param {bool} remove
     * @param {bool} refund
     */
    function GrantOrRemoveAllUpgrades(remove, refund);

    /**
     * Currently holding an item (e.g. capture flag)?
     * @returns {bool}
     */
    function HasItem();

    /**
     * Spoofs a taunt command from the player.
     * @param {integer} taunt_slot
     */
    function HandleTauntCommand(taunt_slot);

    /**
     * @returns {bool}
     */
    function InAirDueToExplosion();

    /**
     * @returns {bool}
     */
    function InAirDueToKnockback();

    /**
     * @param {integer} cond See [Constants.ETFCond](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFCond)
     * @returns {bool}
     */
    function InCond(cond);

    /**
     * @returns {bool}
     */
    function IsAirDashing();

    /**
     * Returns true if the taunt will be stopped.
     * @returns {bool}
     */
    function IsAllowedToRemoveTaunt();

    /**
     * @returns {bool}
     */
    function IsAllowedToTaunt();

    /**
     * Returns true if the player matches this bot type.
     * @param {integer} type See [Constants.EBotType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#EBotType)
     * @returns {bool}
     */
    function IsBotOfType(type);

    /**
     * Is this player calling for medic?
     * @returns {bool}
     */
    function IsCallingForMedic();

    /**
     * @returns {bool}
     */
    function IsCarryingRune();

    /**
     * @returns {bool}
     */
    function IsControlStunned();

    /**
     * @returns {bool}
     */
    function IsCritBoosted();

    /**
     * Returns true if the player is a puppet or AI bot.
     * @returns {bool}
     */
    function IsFakeClient();

    /**
     * @returns {bool}
     */
    function IsFireproof();

    /**
     * @returns {bool}
     */
    function IsFullyInvisible();

    /**
     * @returns {bool}
     */
    function IsHypeBuffed();

    /**
     * @returns {bool}
     */
    function IsImmuneToPushback();

    /**
     * @returns {bool}
     */
    function IsInspecting();

    /**
     * @returns {bool}
     */
    function IsInvulnerable();

    /**
     * @returns {bool}
     */
    function IsJumping();

    /**
     * Is this player an MvM mini-boss?
     * @returns {bool}
     */
    function IsMiniBoss();

    /**
     * @returns {bool}
     */
    function IsParachuteEquipped();

    /**
     * Returns true if we placed a sapper in the last few moments.
     * @returns {bool}
     */
    function IsPlacingSapper();

    /**
     * @returns {bool}
     */
    function IsRageDraining();

    /**
     * @returns {bool}
     */
    function IsRegenerating();

    /**
     * Returns true if we are currently sapping.
     * @returns {bool}
     */
    function IsSapping();

    /**
     * @returns {bool}
     */
    function IsSnared();

    /**
     * @returns {bool}
     */
    function IsStealthed();

    /**
     * @returns {bool}
     */
    function IsTaunting();

    /**
     * @returns {bool}
     */
    function IsUsingActionSlot();

    /**
     * @returns {bool}
     */
    function IsViewingCYOAPDA();

    /**
     * Resupplies a player. If refill_health_ammo is set, clears negative conds and gives health/ammo.
     * @param {bool} refill_health_ammo
     */
    function Regenerate(refill_health_ammo);

    /**
     * Remove all player objects (e.g. dispensers/sentries).
     * @param {bool} explode
     */
    function RemoveAllObjects(explode);

    /**
     * Removes a condition.
     * @param {integer} cond See [Constants.ETFCond](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFCond)
     */
    function RemoveCond(cond);

    /**
     * Extended version of RemoveCond. Allows forcefully removing the condition.
     * @param {integer} cond See [Constants.ETFCond](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFCond)
     * @param {bool} ignore_duration
     */
    function RemoveCondEx(cond, ignore_duration);

    /**
     * Take away money from a player. Lower bounded to 0.
     * @param {integer} amount
     */
    function RemoveCurrency(amount);

    /**
     * Remove a custom attribute from the player.
     * @param {attribute} name
     */
    function RemoveCustomAttribute(name);

    /**
     * Undisguise a spy.
     */
    function RemoveDisguise();

    /**
     * Unhides a HUD element(s).
     * @param {integer} flags See [Constants.FHideHUD](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FHideHUD)
     */
    function RemoveHudHideFlags(flags);

    /**
     * Un-invisible a spy.
     */
    function RemoveInvisibility();

    /**
     */
    function RemoveTeleportEffect();

    /**
     */
    function ResetScores();

    /**
     */
    function RollRareSpell();

    /**
     * @param {integer} cond See [Constants.ETFCond](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFCond)
     * @param {float} duration
     */
    function SetCondDuration(cond, duration);

    /**
     * Set player's cash for MvM. Does not have any bounds checking.
     * @param {integer} amount
     */
    function SetCurrency(amount);

    /**
     * @param {float} speed
     */
    function SetCurrentTauntMoveSpeed(speed);

    /**
     * Sets a custom player model without animations (model will T-pose).
     * @param {string} model_name
     */
    function SetCustomModel(model_name);

    /**
     * @param {Vector} offset
     */
    function SetCustomModelOffset(offset);

    /**
     * @param {bool} toggle
     */
    function SetCustomModelRotates(toggle);

    /**
     * @param {QAngle} angles
     */
    function SetCustomModelRotation(angles);

    /**
     * @param {bool} toggle
     */
    function SetCustomModelVisibleToSelf(toggle);

    /**
     * Sets a custom player model with full animations.
     * @param {string} model_name
     */
    function SetCustomModelWithClassAnimations(model_name);

    /**
     * @param {integer} count
     */
    function SetDisguiseAmmoCount(count);

    /**
     * @param {integer} toggle
     */
    function SetForcedTauntCam(toggle);

    /**
     * Set the player's target grapple entity.
     * @param {CBaseEntity|null} entity
     * @param {bool} bleed
     */
    function SetGrapplingHookTarget(entity, bleed);

    /**
     * Force HUD hide flags to a value.
     * @param {integer} flags See [Constants.FHideHUD](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FHideHUD)
     */
    function SetHudHideFlags(flags);

    /**
     * Make this player an MvM mini-boss.
     * @param {bool} toggle
     */
    function SetIsMiniBoss(toggle);

    /**
     * Set next change class time.
     * @param {float} time
     */
    function SetNextChangeClassTime(time);

    /**
     * Set next change team time.
     * @param {float} time
     */
    function SetNextChangeTeamTime(time);

    /**
     * Set next available resupply time.
     * @param {float} time
     */
    function SetNextRegenTime(time);

    /**
     * Sets the player class. Updates the player's visuals and model.
     * @param {integer} class_index See [Constants.ETFClass](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFClass)
     */
    function SetPlayerClass(class_index);

    /**
     * Sets rage meter from 0 - 100.
     * @param {float} percent
     */
    function SetRageMeter(percent);

    /**
     * Rig the result of Rock-Paper-Scissors (0=rock, 1=paper, 2=scissors).
     * @param {integer} result
     */
    function SetRPSResult(result);

    /**
     * Sets hype meter from 0 - 100.
     * @param {float} percent
     */
    function SetScoutHypeMeter(percent);

    /**
     * Sets cloakmeter from 0 - 100.
     * @param {float} percent
     */
    function SetSpyCloakMeter(percent);

    /**
     * Set the timestamp when kart was reversed.
     * @param {float} time
     */
    function SetVehicleReverseTime(time);

    /**
     * @param {bool} toggle
     */
    function SetUseBossHealthBar(toggle);

    /**
     * Stops current taunt.
     * @param {bool} remove_prop
     */
    function StopTaunt(remove_prop);

    /**
     * Stuns the player for a specified duration.
     * @param {float} duration
     * @param {float} move_speed_reduction
     * @param {integer} flags See [Constants.TF_STUN](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#TF_STUN)
     * @param {CBaseEntity|null} attacker
     */
    function StunPlayer(duration, move_speed_reduction, flags, attacker);

    /**
     * Performs a taunt if allowed.
     * @param {integer} taunt_index See [Constants.FTaunts](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FTaunts)
     * @param {integer} taunt_concept See [Constants.MP_CONCEPT](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#MP_CONCEPT)
     */
    function Taunt(taunt_index, taunt_concept);

    /**
     * Make the player attempt to pick up a building in front of them.
     * @returns {bool}
     */
    function TryToPickupBuilding();

    /**
     * @param {integer} skin
     */
    function UpdateSkin(skin);

    /**
     * @param {integer} cond See [Constants.ETFCond](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFCond)
     * @returns {bool}
     */
    function WasInCond(cond);

    /**
     * @param {CTFWeaponBase} weapon
     * @returns {bool}
     */
    function Weapon_CanUse(weapon);

    /**
     * Equips a weapon in the player. Places it inside the m_hMyWeapons array.
     * @param {CTFWeaponBase} weapon
     */
    function Weapon_Equip(weapon);

    /**
     * @param {CTFWeaponBase} weapon
     */
    function Weapon_SetLast(weapon);

    /**
     * The same as calling EyePosition.
     * @returns {Vector}
     */
    function Weapon_ShootPosition();

    /**
     * Attempts a switch to the given weapon, if present in the player's inventory.
     * @param {CTFWeaponBase} weapon
     */
    function Weapon_Switch(weapon);
}

// ============================================================
// CTFBot extends CTFPlayer
// ============================================================

/**
 * Script handle class for bot-controlled players (tf_bot).
 *
 * **Note**: Puppet bots do NOT inherit from this class.
 * @extends {CTFPlayer | NextBotCombatCharacter}
 */
class CTFBot extends CTFPlayer {
    /**
     * Sets attribute flags on this TFBot.
     * @param {integer} attribute See [Constants.FTFBotAttributeType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FTFBotAttributeType)
     */
    function AddBotAttribute(attribute);

    /**
     * Adds a bot tag.
     * @param {string} tag
     */
    function AddBotTag(tag);

    /**
     * Adds weapon restriction flags.
     * @param {integer} flags See [Constants.TFBotWeaponRestrictionType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#TFBotWeaponRestrictionType)
     */
    function AddWeaponRestriction(flags);

    /**
     * Clears all attribute flags on this TFBot.
     */
    function ClearAllBotAttributes();

    /**
     * Clears bot tags.
     */
    function ClearAllBotTags();

    /**
     * Removes all weapon restriction flags.
     */
    function ClearAllWeaponRestrictions();

    /**
     * Clear current focus.
     */
    function ClearAttentionFocus();

    /**
     * Clear the given behavior flag(s) for this bot.
     * @param {integer} flags See [Constants.TFBOT_BEHAVIOR](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#TFBOT_BEHAVIOR)
     */
    function ClearBehaviorFlag(flags);

    /**
     * Notice the threat after a delay in seconds.
     * @param {CBaseEntity} threat
     * @param {float} delay
     */
    function DelayedThreatNotice(threat, delay);

    /**
     * Forces the current squad to be entirely disbanded by everyone.
     */
    function DisbandCurrentSquad();

    /**
     * Get the nav area of the closest vantage point (within distance).
     * @param {float} max_distance
     * @returns {CTFNavArea|null}
     */
    function FindVantagePoint(max_distance);

    /**
     * Give me an item!
     * @param {string} item_name
     */
    function GenerateAndWearItem(item_name);

    /**
     * Get the given action point for this bot.
     * @returns {CBaseEntity|null}
     */
    function GetActionPoint();

    /**
     * Get all bot tags. The key is the index, and the value is the tag.
     * @param {table} result
     */
    function GetAllBotTags(result);

    /**
     * Gets the home nav area of the bot.
     * @returns {CTFNavArea|null}
     */
    function GetHomeArea();

    /**
     * Returns the bot's difficulty level.
     * @returns {integer} See [Constants.ETFBotDifficultyType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFBotDifficultyType)
     */
    function GetDifficulty();

    /**
     * Gets the max vision range override for the bot.
     * @returns {float}
     */
    function GetMaxVisionRangeOverride();

    /**
     * Get this bot's current mission.
     * @returns {integer} See [Constants.ETFBotMissionType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFBotMissionType)
     */
    function GetMission();

    /**
     * Get this bot's current mission target.
     * @returns {CBaseEntity|null}
     */
    function GetMissionTarget();

    /**
     * Gets the nearest known sappable target.
     * @returns {CBaseEntity|null}
     */
    function GetNearestKnownSappableTarget();

    /**
     * Get this bot's previous mission.
     * @returns {integer} See [Constants.ETFBotMissionType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFBotMissionType)
     */
    function GetPrevMission();

    /**
     * Return the nav area of where we spawned.
     * @returns {CTFNavArea|null}
     */
    function GetSpawnArea();

    /**
     * Gets our formation error coefficient.
     * @returns {float}
     */
    function GetSquadFormationError();

    /**
     * Checks if this TFBot has the given attributes.
     * @param {integer} attribute See [Constants.FTFBotAttributeType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FTFBotAttributeType)
     * @returns {bool}
     */
    function HasBotAttribute(attribute);

    /**
     * Checks if this TFBot has the given bot tag.
     * @param {string} tag
     * @returns {bool}
     */
    function HasBotTag(tag);

    /**
     * Return true if the given mission is this bot's current mission.
     * @param {integer} mission See [Constants.ETFBotMissionType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFBotMissionType)
     * @returns {bool}
     */
    function HasMission(mission);

    /**
     * Checks if this TFBot has the given weapon restriction flags.
     * @param {integer} flags See [Constants.TFBotWeaponRestrictionType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#TFBotWeaponRestrictionType)
     * @returns {bool}
     */
    function HasWeaponRestriction(flags);

    /**
     * @returns {bool}
     */
    function IsAmmoFull();

    /**
     * @returns {bool}
     */
    function IsAmmoLow();

    /**
     * Is our attention focused right now?
     * @returns {bool}
     */
    function IsAttentionFocused();

    /**
     * Is our attention focused on this entity.
     * @param {CBaseEntity} entity
     * @returns {bool}
     */
    function IsAttentionFocusedOn(entity);

    /**
     * Return true if the given behavior flag(s) are set for this bot.
     * @param {integer} flags See [Constants.TFBOT_BEHAVIOR](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#TFBOT_BEHAVIOR)
     * @returns {bool}
     */
    function IsBehaviorFlagSet(flags);

    /**
     * Returns true/false if the bot's difficulty level matches.
     * @param {integer} difficulty See [Constants.ETFBotDifficultyType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFBotDifficultyType)
     * @returns {bool}
     */
    function IsDifficulty(difficulty);

    /**
     * Checks if we are in a squad.
     * @returns {bool}
     */
    function IsInASquad();

    /**
     * Return true if this bot has a current mission.
     * @returns {bool}
     */
    function IsOnAnyMission();

    /**
     * Checks if the given weapon is restricted for use on the bot.
     * @param {CBaseEntity} weapon
     * @returns {bool}
     */
    function IsWeaponRestricted(weapon);

    /**
     * Makes us leave the current squad (if any).
     */
    function LeaveSquad();

    /**
     * @param {float} duration
     */
    function PressAltFireButton(duration = -1.0);

    /**
     * @param {float} duration
     */
    function PressFireButton(duration = -1.0);

    /**
     * @param {float} duration
     */
    function PressSpecialFireButton(duration = -1.0);

    /**
     * Removes attribute flags on this TFBot.
     * @param {integer} attribute See [Constants.FTFBotAttributeType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FTFBotAttributeType)
     */
    function RemoveBotAttribute(attribute);

    /**
     * Removes a bot tag.
     * @param {string} tag
     */
    function RemoveBotTag(tag);

    /**
     * Removes weapon restriction flags.
     * @param {integer} flags See [Constants.TFBotWeaponRestrictionType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#TFBotWeaponRestrictionType)
     */
    function RemoveWeaponRestriction(flags);

    /**
     * Set the given action point for this bot.
     * @param {CBaseEntity|null} entity
     */
    function SetActionPoint(entity);

    /**
     * Sets our current attention focus to this entity.
     * @param {CBaseEntity|null} entity
     */
    function SetAttentionFocus(entity);

    /**
     * Sets if the bot should automatically jump, and how often.
     * @param {float} min_time
     * @param {float} max_time
     */
    function SetAutoJump(min_time, max_time);

    /**
     * Set the given behavior flag(s) for this bot.
     * @param {integer} flags See [Constants.TFBOT_BEHAVIOR](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#TFBOT_BEHAVIOR)
     */
    function SetBehaviorFlag(flags);

    /**
     * Sets the bots difficulty level.
     * @param {integer} difficulty See [Constants.ETFBotDifficultyType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFBotDifficultyType)
     */
    function SetDifficulty(difficulty);

    /**
     * Set the home nav area of the bot.
     * @param {CTFNavArea|null} area
     */
    function SetHomeArea(area);

    /**
     * Sets max vision range override for the bot.
     * @param {float} range
     */
    function SetMaxVisionRangeOverride(range);

    /**
     * Set this bot's current mission to the given mission.
     * @param {integer} mission See [Constants.ETFBotMissionType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFBotMissionType)
     * @param {bool} reset_behavior
     */
    function SetMission(mission, reset_behavior);

    /**
     * Set this bot's mission target to the given entity.
     * @param {CBaseEntity|null} entity
     */
    function SetMissionTarget(entity);

    /**
     * Set this bot's previous mission to the given mission.
     * @param {integer} mission See [Constants.ETFBotMissionType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFBotMissionType)
     */
    function SetPrevMission(mission);

    /**
     * Sets the scale override for the bot.
     * @param {float} scale
     */
    function SetScaleOverride(scale);

    /**
     * Sets if the bot should build instantly.
     * @param {bool} toggle
     */
    function SetShouldQuickBuild(toggle);

    /**
     * Sets our formation error coefficient.
     * @param {float} coefficient
     */
    function SetSquadFormationError(coefficient);

    /**
     * Returns if the bot should automatically jump.
     * @returns {bool}
     */
    function ShouldAutoJump();

    /**
     * Returns if the bot should build instantly.
     * @returns {bool}
     */
    function ShouldQuickBuild();

    /**
     */
    function UpdateDelayedThreatNotices();


    // Another multiple inheritance
    // From NextBotCombatCharacter
    /**
     * Clear immobile status.
     */
    function ClearImmobileStatus();

    /**
     * Flag this bot for update.
     * Tip: Use in think function to update nextbots faster than nb_update_frequency.
     * @param {bool} toggle
     */
    function FlagForUpdate(toggle);

    /**
     * Get this bot's body interface.
     * @returns {INextBotComponent}
     */
    function GetBodyInterface();

    /**
     * Get this bot's id.
     * @returns {integer}
     */
    function GetBotId();

    /**
     * How long have we been immobile.
     * @returns {float}
     */
    function GetImmobileDuration();

    /**
     * Return units/second below which this actor is considered immobile.
     * @returns {float}
     */
    function GetImmobileSpeedThreshold();

    /**
     * Get this bot's intention interface.
     * @returns {INextBotComponent}
     */
    function GetIntentionInterface();

    /**
     * Get this bot's locomotion interface.
     * @returns {ILocomotion}
     */
    function GetLocomotionInterface();

    /**
     * Get last update tick.
     * @returns {integer}
     */
    function GetTickLastUpdate();

    /**
     * Get this bot's vision interface.
     * @returns {INextBotComponent}
     */
    function GetVisionInterface();

    /**
     * Return true if given entity is our enemy.
     * @param {CBaseEntity} entity
     * @returns {bool}
     */
    function IsEnemy(entity);

    /**
     * Is this bot flagged for update.
     * @returns {bool}
     */
    function IsFlaggedForUpdate();

    /**
     * Return true if given entity is our friend.
     * @param {CBaseEntity} entity
     * @returns {bool}
     */
    function IsFriend(entity);

    /**
     * Return true if we haven't moved in awhile.
     * @returns {bool}
     */
    function IsImmobile();
}

// ============================================================
// CTFBaseBoss extends NextBotCombatCharacter
// ============================================================

/**
 * Base class intended for custom NPCs. Officially used as part of MvM tank.
 */
class CTFBaseBoss extends NextBotCombatCharacter {
    /**
     * Sets whether the entity should push away players intersecting its bounding box. On by default.
     * @param {bool} toggle
     */
    function SetResolvePlayerCollisions(toggle);
}

// ============================================================
// Convars (Game Instance)
// ============================================================

/**
 * An interface to manipulate the convars on the server.
 *
 * **Note**: Protected convars (e.g. `rcon_password`) cannot be accessed.
 */
class Convars {
    /**
     * Returns the convar as a bool. May return null if no such convar.
     * @param {convar} name
     * @returns {bool|null}
     */
    function GetBool(name);

    /**
     * Returns the convar value for the entindex as a string. Only works on FCVAR_USERINFO convars.
     * @param {client_convar} name
     * @param {integer} entindex
     * @returns {string}
     */
    function GetClientConvarValue(name, entindex);

    /**
     * Returns the convar as an int. May return null if no such convar.
     *
     * **Warning**: The entire convar list is searched each time (slow). Cache results if used often.
     * @param {convar} name
     * @returns {integer|null}
     */
    function GetInt(name);

    /**
     * Returns the convar as a string. May return null if no such convar.
     *
     * **Warning**: The entire convar list is searched each time (slow). Cache results if used often.
     * @param {convar} name
     * @returns {string|null}
     */
    function GetStr(name);

    /**
     * Returns the convar as a float. May return null if no such convar.
     *
     * **Warning**: The entire convar list is searched each time (slow). Cache results if used often.
     * @param {convar} name
     * @returns {float|null}
     */
    function GetFloat(name);

    /**
     * Checks if the convar is allowed to be used (in cfg/vscript_convar_allowlist.txt).
     * @param {convar} name
     * @returns {bool}
     */
    function IsConVarOnAllowList(name);

    /**
     * Sets the value of the convar. The convar must be in cfg/vscript_convar_allowlist.txt.
     * The original value is saved and reset on map change.
     * @param {convar} name
     * @param {any} value
     */
    function SetValue(name, value);
}

// ============================================================
// CEntities (Game Instance: Entities)
// ============================================================

/**
 * An interface to find and iterate over the script handles for the entities in play.
 * Pass null to the previous parameter to start an iteration.
 */
class CEntities {
    /**
     * Creates an entity by classname. Returns null if no entity type could be inferred.
     * @param {classname} classname
     * @returns {CBaseEntity|null}
     */
    function CreateByClassname(classname);

    /**
     * Dispatches spawn of an entity. Use this on entities created via CreateByClassname.
     * @param {CBaseEntity} entity
     */
    function DispatchSpawn(entity);

    /**
     * Find entities by classname. Pass null to start, or previous entity to continue.
     * @param {CBaseEntity|null} previous
     * @param {classname_search} classname
     * @returns {CBaseEntity|null}
     */
    function FindByClassname(previous, classname);

    /**
     * Find entities by classname nearest to a point within a radius.
     * @param {classname_search} classname
     * @param {Vector} center
     * @param {float} radius
     * @returns {CBaseEntity|null}
     */
    function FindByClassnameNearest(classname, center, radius);

    /**
     * Find entities by classname within a radius. Pass null to start, or previous to continue.
     * @param {CBaseEntity|null} previous
     * @param {classname_search} classname
     * @param {Vector} center
     * @param {float} radius
     * @returns {CBaseEntity|null}
     */
    function FindByClassnameWithin(previous, classname, center, radius);

    /**
     * Find entities by model keyvalue. Pass null to start, or previous to continue.
     * @param {CBaseEntity|null} previous
     * @param {string} model_name
     * @returns {CBaseEntity|null}
     */
    function FindByModel(previous, model_name);

    /**
     * Find entities by targetname keyvalue. Pass null to start, or previous to continue.
     * @param {CBaseEntity|null} previous
     * @param {string} targetname
     * @returns {CBaseEntity|null}
     */
    function FindByName(previous, targetname);

    /**
     * Find entities by targetname nearest to a point within a radius.
     * @param {string} targetname
     * @param {Vector} center
     * @param {float} radius
     * @returns {CBaseEntity|null}
     */
    function FindByNameNearest(targetname, center, radius);

    /**
     * Find entities by targetname within a radius. Pass null to start, or previous to continue.
     * @param {CBaseEntity|null} previous
     * @param {string} targetname
     * @param {Vector} center
     * @param {float} radius
     * @returns {CBaseEntity|null}
     */
    function FindByNameWithin(previous, targetname, center, radius);

    /**
     * Find entities by their target keyvalue. Pass null to start, or previous to continue.
     * @param {CBaseEntity|null} previous
     * @param {string} target
     * @returns {CBaseEntity|null}
     */
    function FindByTarget(previous, target);

    /**
     * Find entities within a radius. Pass null to start, or previous to continue.
     * @param {CBaseEntity|null} previous
     * @param {Vector} center
     * @param {float} radius
     * @returns {CBaseEntity|null}
     */
    function FindInSphere(previous, center, radius);

    /**
     * Begin an iteration over the list of entities. The first entity is always worldspawn.
     * @returns {CBaseEntity}
     */
    function First();

    /**
     * Returns the next entity after the given one in the list.
     * @param {CBaseEntity} previous
     * @returns {CBaseEntity|null}
     */
    function Next(previous);
}

// ============================================================
// CTFNavArea
// ============================================================

/**
 * Script handle class for areas part of the navigation mesh.
 */
class CTFNavArea {
    /**
     * Add areas that connect TO this area by a ONE-WAY link.
     * @param {CTFNavArea} area
     * @param {integer} dir See [Constants.ENavDirType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ENavDirType)
     */
    function AddIncomingConnection(area, dir);

    /**
     * Clear TF-specific area attribute bits.
     * @param {integer} bits See [Constants.FTFNavAttributeType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FTFNavAttributeType)
     */
    function ClearAttributeTF(bits);

    /**
     * Compute closest point within the portal between areas.
     * @param {CTFNavArea} to
     * @param {integer} dir See [Constants.ENavDirType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ENavDirType)
     * @param {Vector} close_pos
     * @returns {Vector}
     */
    function ComputeClosestPointInPortal(to, dir, close_pos);

    /**
     * Return direction from this area to the given point.
     * @param {Vector} point
     * @returns {integer}
     */
    function ComputeDirection(point);

    /**
     * Connect this area to given area in given direction.
     * @param {CTFNavArea} area
     * @param {integer} dir See [Constants.ENavDirType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ENavDirType)
     */
    function ConnectTo(area, dir);

    /**
     * Return true if other area is on or above this area, but no others.
     * @param {CTFNavArea} area
     * @returns {bool}
     */
    function Contains(area);

    /**
     * Return true if given point is on or above this area, but no others.
     * @param {Vector} point
     * @returns {bool}
     */
    function ContainsOrigin(point);

    /**
     * Draw area as a filled rectangle of the given color.
     * @param {integer} r
     * @param {integer} g
     * @param {integer} b
     * @param {integer} a
     * @param {float} duration
     * @param {bool} no_depth_test
     * @param {float} margin
     */
    function DebugDrawFilled(r, g, b, a, duration, no_depth_test, margin);

    /**
     * Disconnect this area from given area.
     * @param {CTFNavArea} area
     */
    function Disconnect(area);

    /**
     * Get random origin within extent of area.
     * @returns {Vector}
     */
    function FindRandomSpot();

    /**
     * Return the n'th adjacent area in the given direction.
     * @param {integer} dir See [Constants.ENavDirType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ENavDirType)
     * @param {integer} n
     * @returns {CTFNavArea|null}
     */
    function GetAdjacentArea(dir, n);

    /**
     * Fills a passed in table with all adjacent areas in the given direction.
     * @param {integer} dir See [Constants.ENavDirType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ENavDirType)
     * @param {table} result
     */
    function GetAdjacentAreas(dir, result);

    /**
     * Get the number of adjacent areas in the given direction.
     * @param {integer} dir See [Constants.ENavDirType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ENavDirType)
     * @returns {integer}
     */
    function GetAdjacentCount(dir);

    /**
     * Get area attribute bits.
     * @returns {integer} See [Constants.FNavAttributeType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FNavAttributeType)
     */
    function GetAttributes();

    /**
     * Returns the maximum height of the obstruction above the ground.
     * @returns {float}
     */
    function GetAvoidanceObstacleHeight();

    /**
     * Get center origin of area.
     * @returns {Vector}
     */
    function GetCenter();

    /**
     * Get corner origin of area.
     * @param {integer} dir See [Constants.ENavDirType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ENavDirType)
     * @returns {Vector}
     */
    function GetCorner(dir);

    /**
     * Return shortest distance between point and this area.
     * @param {Vector} pos
     * @returns {float}
     */
    function GetDistanceSquaredToPoint(pos);

    /**
     * Returns the door entity above the area.
     * @returns {CBaseAnimating|null}
     */
    function GetDoor();

    /**
     * Returns the elevator if in an elevator's path.
     * @returns {CBaseAnimating|null}
     */
    function GetElevator();

    /**
     * Fills table with a collection of areas reachable via elevator from this area.
     * @param {table} result
     */
    function GetElevatorAreas(result);

    /**
     * Get area ID.
     * @returns {integer}
     */
    function GetID();

    /**
     * Fills a passed in table with areas connected TO this area by a ONE-WAY link.
     * @param {integer} dir See [Constants.ENavDirType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ENavDirType)
     * @param {table} result
     */
    function GetIncomingConnections(dir, result);

    /**
     * Returns the area just prior to this one in the search path.
     * @returns {CTFNavArea|null}
     */
    function GetParent();

    /**
     * Returns how we get from parent to us.
     * @returns {integer}
     */
    function GetParentHow();

    /**
     * Get place name if it exists, null otherwise.
     * @returns {string|null}
     */
    function GetPlaceName();

    /**
     * Return number of players of given team currently within this area (0 = any/all).
     * @param {integer} team See [Constants.ETFTeam](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFTeam)
     * @returns {integer}
     */
    function GetPlayerCount(team);

    /**
     * Return a random adjacent area in the given direction.
     * @param {integer} dir See [Constants.ENavDirType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ENavDirType)
     * @returns {CTFNavArea|null}
     */
    function GetRandomAdjacentArea(dir);

    /**
     * Return the area size along the X axis.
     * @returns {float}
     */
    function GetSizeX();

    /**
     * Return the area size along the Y axis.
     * @returns {float}
     */
    function GetSizeY();

    /**
     * Gets the travel distance to the MvM bomb target.
     * @returns {float}
     */
    function GetTravelDistanceToBombTarget();

    /**
     * Return Z of area at (x,y) of 'pos'.
     * @param {Vector} pos
     * @returns {float}
     */
    function GetZ(pos);

    /**
     * Has TF-specific area attribute bits of the given ones.
     * @param {integer} bits See [Constants.FTFNavAttributeType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FTFNavAttributeType)
     * @returns {bool}
     */
    function HasAttributeTF(bits);

    /**
     * Has area attribute bits of the given ones.
     * @param {integer} bits See [Constants.FNavAttributeType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FNavAttributeType)
     * @returns {bool}
     */
    function HasAttributes(bits);

    /**
     * Returns true if there's a large, immobile object obstructing this area.
     * @param {float} max_height
     * @returns {bool}
     */
    function HasAvoidanceObstacle(max_height);

    /**
     * Return true if team is blocked in this area.
     * @param {integer} team See [Constants.ETFTeam](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFTeam)
     * @param {bool} affects_flow
     * @returns {bool}
     */
    function IsBlocked(team, affects_flow);

    /**
     * Returns true if area is a bottleneck.
     * @returns {bool}
     */
    function IsBottleneck();

    /**
     * Return true if given area is completely visible from somewhere in this area.
     * @param {integer} team See [Constants.ETFTeam](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFTeam)
     * @returns {bool}
     */
    function IsCompletelyVisibleToTeam(team);

    /**
     * Return true if this area is connected to other area in given direction.
     * @param {CBaseEntity} area
     * @param {integer} dir See [Constants.ENavDirType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ENavDirType)
     * @returns {bool}
     */
    function IsConnected(area, dir);

    /**
     * Return true if this area and given area are approximately co-planar.
     * @param {CBaseEntity} area
     * @returns {bool}
     */
    function IsCoplanar(area);

    /**
     * Return true if this area is marked to have continuous damage.
     * @returns {bool}
     */
    function IsDamaging();

    /**
     * Return true if this area is badly formed.
     * @returns {bool}
     */
    function IsDegenerate();

    /**
     * Return true if there are no bi-directional links on the given side.
     * @param {integer} dir See [Constants.ENavDirType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ENavDirType)
     * @returns {bool}
     */
    function IsEdge(dir);

    /**
     * Return true if this area is approximately flat.
     * @returns {bool}
     */
    function IsFlat();

    /**
     * Return true if 'area' overlaps our 2D extents.
     * @param {CBaseEntity} area
     * @returns {bool}
     */
    function IsOverlapping(area);

    /**
     * Return true if 'pos' is within 2D extents of area.
     * @param {Vector} pos
     * @param {float} tolerance
     * @returns {bool}
     */
    function IsOverlappingOrigin(pos, tolerance);

    /**
     * Return true if any portion of this area is visible to anyone on the given team.
     * @param {integer} team See [Constants.ETFTeam](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFTeam)
     * @returns {bool}
     */
    function IsPotentiallyVisibleToTeam(team);

    /**
     * Is this area reachable by the given team?
     * @param {integer} team See [Constants.ETFTeam](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFTeam)
     * @returns {bool}
     */
    function IsReachableByTeam(team);

    /**
     * Return true if this area is approximately square.
     * @returns {bool}
     */
    function IsRoughlySquare();

    /**
     * Is this nav area marked with the current marking scope?
     * @returns {bool}
     */
    function IsTFMarked();

    /**
     * Return true if area is underwater.
     * @returns {bool}
     */
    function IsUnderwater();

    /**
     * Returns true if area is valid for wandering population.
     * @returns {bool}
     */
    function IsValidForWanderingPopulation();

    /**
     * Return true if area is visible from the given eyepoint.
     * @param {Vector} point
     * @returns {bool}
     */
    function IsVisible(point);

    /**
     * Mark this area as blocked for team.
     * @param {integer} team See [Constants.ETFTeam](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFTeam)
     */
    function MarkAsBlocked(team);

    /**
     * Mark this area is damaging for the next 'duration' seconds.
     * @param {float} duration
     */
    function MarkAsDamaging(duration);

    /**
     * Marks the obstructed status of the nav area.
     * @param {float} height
     */
    function MarkObstacleToAvoid(height);

    /**
     * Removes area attribute bits.
     * @param {integer} bits See [Constants.FNavAttributeType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FNavAttributeType)
     */
    function RemoveAttributes(bits);

    /**
     * Removes all connections in directions to left and right of specified direction.
     * @param {integer} dir See [Constants.ENavDirType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ENavDirType)
     */
    function RemoveOrthogonalConnections(dir);

    /**
     * Set TF-specific area attributes.
     * @param {integer} bits See [Constants.FTFNavAttributeType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FTFNavAttributeType)
     */
    function SetAttributeTF(bits);

    /**
     * Set area attribute bits.
     * @param {integer} bits See [Constants.FNavAttributeType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FNavAttributeType)
     */
    function SetAttributes(bits);

    /**
     * Set place name. Pass null to clear.
     * @param {string} name
     */
    function SetPlaceName(name);

    /**
     * Mark this nav area with the current marking scope.
     */
    function TFMark();

    /**
     * Unblocks this area.
     */
    function UnblockArea();
}

// ============================================================
// CNavMesh (Game Instance: NavMesh)
// ============================================================

/**
 * An interface to collect nav areas from, especially for pathfinding needs.
 */
class CNavMesh {
    /**
     * Get nav area from ray.
     * @param {Vector} start_pos
     * @param {Vector} end_pos
     * @param {CTFNavArea|null} ignore_area
     * @returns {CTFNavArea|null}
     */
    function FindNavAreaAlongRay(start_pos, end_pos, ignore_area);

    /**
     * Fills a passed in table of all nav areas.
     * @param {table} result Resulting shape: {"area0": CTFNavArea, "area1": CTFNavArea, ...}
     */
    function GetAllAreas(result);

    /**
     * Fills a passed in table of all nav areas that have the specified attributes.
     * @param {integer} bits See [Constants.FNavAttributeType](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FNavAttributeType)
     * @param {table} result
     */
    function GetAreasWithAttributes(bits, result);

    /**
     * Given a position in the world, return the nav area closest to or below that height.
     * @param {Vector} origin
     * @param {float} beneath
     * @returns {CTFNavArea|null}
     */
    function GetNavArea(origin, beneath);

    /**
     * Get nav area by ID.
     * @param {integer} area_id
     * @returns {CTFNavArea|null}
     */
    function GetNavAreaByID(area_id);

    /**
     * Return total number of nav areas.
     * @returns {integer}
     */
    function GetNavAreaCount();

    /**
     * Fills the table with areas from a path. Returns whether a path was found.
     *
     * **Note**: The areas are passed from end area to the start area.
     * @param {CTFNavArea} start_area
     * @param {CTFNavArea} end_area
     * @param {Vector} goal_pos
     * @param {float} max_path_length
     * @param {integer} team See [Constants.ETFTeam](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFTeam)
     * @param {bool} ignore_nav_blockers
     * @param {table} result
     * @returns {bool}
     */
    function GetNavAreasFromBuildPath(start_area, end_area, goal_pos, max_path_length, team, ignore_nav_blockers, result);

    /**
     * Fills a passed in table of nav areas within radius.
     * @param {Vector} origin
     * @param {float} radius
     * @param {table} result
     */
    function GetNavAreasInRadius(origin, radius, result);

    /**
     * Fills passed in table with areas overlapping entity's extent.
     * @param {CBaseEntity} entity
     * @param {table} result
     */
    function GetNavAreasOverlappingEntityExtent(entity, result);

    /**
     * Given a position in the world, return the nav area closest to or below that height.
     * @param {Vector} origin
     * @param {float} max_distance
     * @param {bool} check_los
     * @param {bool} check_ground
     * @returns {CTFNavArea|null}
     */
    function GetNearestNavArea(origin, max_distance, check_los, check_ground);

    /**
     * Fills a passed in table of all obstructing entities.
     * @param {table} result
     */
    function GetObstructingEntities(result);

    /**
     * Returns true if a path exists.
     * @param {CTFNavArea} start_area
     * @param {CTFNavArea} end_area
     * @param {Vector} goal_pos
     * @param {float} max_path_length
     * @param {integer} team See [Constants.ETFTeam](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFTeam)
     * @param {bool} ignore_nav_blockers
     * @returns {bool}
     */
    function NavAreaBuildPath(start_area, end_area, goal_pos, max_path_length, team, ignore_nav_blockers);

    /**
     * Compute distance between two areas. Returns -1.0 if can't reach 'end_area' from 'start_area'.
     * @param {CTFNavArea} start_area
     * @param {CTFNavArea} end_area
     * @param {float} max_path_length
     * @returns {float}
     */
    function NavAreaTravelDistance(start_area, end_area, max_path_length);

    /**
     * Registers avoidance obstacle.
     * @param {CBaseEntity} entity
     */
    function RegisterAvoidanceObstacle(entity);

    /**
     * Unregisters avoidance obstacle.
     * @param {CBaseEntity} entity
     */
    function UnregisterAvoidanceObstacle(entity);
}

// ============================================================
// CNetPropManager (Game Instance: NetProps)
// ============================================================

/**
 * Allows reading and updating the network properties and data-maps of an entity.
 */
class CNetPropManager {
    /**
     * Returns the size of a netprop array, or -1.
     * @param {CBaseEntity} entity
     * @param {property_array} property_name
     * @returns {integer}
     */
    function GetPropArraySize(entity, property_name);

    /**
     * Reads an EHANDLE-valued netprop. Returns null if property is not found.
     * @param {CBaseEntity} entity
     * @param {entity_property} property_name
     * @returns {CBaseEntity|null}
     */
    function GetPropEntity(entity, property_name);

    /**
     * Reads an EHANDLE-valued netprop from an array. Returns null if not found.
     * @param {CBaseEntity} entity
     * @param {entity_array_property} property_name
     * @param {integer} array_element
     * @returns {CBaseEntity|null}
     */
    function GetPropEntityArray(entity, property_name, array_element);

    /**
     * Reads a boolean-valued netprop. Returns false if property is not found.
     * @param {CBaseEntity} entity
     * @param {bool_property} property_name
     * @returns {bool}
     */
    function GetPropBool(entity, property_name);

    /**
     * Reads a boolean-valued netprop from an array. Returns false if not found.
     * @param {CBaseEntity} entity
     * @param {bool_array_property} property_name
     * @param {integer} array_element
     * @returns {bool}
     */
    function GetPropBoolArray(entity, property_name, array_element);

    /**
     * Reads a float-valued netprop. Returns -1.0 if property is not found.
     * @param {CBaseEntity} entity
     * @param {float_property} property_name
     * @returns {float}
     */
    function GetPropFloat(entity, property_name);

    /**
     * Reads a float-valued netprop from an array. Returns -1.0 if not found.
     * @param {CBaseEntity} entity
     * @param {float_array_property} property_name
     * @param {integer} array_element
     * @returns {float}
     */
    function GetPropFloatArray(entity, property_name, array_element);

    /**
     * Fills in a passed table with property info for the provided entity.
     * @param {CBaseEntity} entity
     * @param {property} property_name
     * @param {integer} array_element
     * @param {table} result
     * @returns {bool}
     */
    function GetPropInfo(entity, property_name, array_element, result);

    /**
     * Reads an integer-valued netprop. Returns -1 if property is not found.
     * @param {CBaseEntity} entity
     * @param {integer_property} property_name
     * @returns {integer}
     */
    function GetPropInt(entity, property_name);

    /**
     * Reads an integer-valued netprop from an array. Returns -1 if not found.
     * @param {CBaseEntity} entity
     * @param {integer_array_property} property_name
     * @param {integer} array_element
     * @returns {integer}
     */
    function GetPropIntArray(entity, property_name, array_element);

    /**
     * Reads a string-valued netprop. Returns empty string if property is not found.
     * @param {CBaseEntity} entity
     * @param {string_property} property_name
     * @returns {string}
     */
    function GetPropString(entity, property_name);

    /**
     * Reads a string-valued netprop from an array. Returns empty string if not found.
     * @param {CBaseEntity} entity
     * @param {string_array_property} property_name
     * @param {integer} array_element
     * @returns {string}
     */
    function GetPropStringArray(entity, property_name, array_element);

    /**
     * Returns the name of the netprop type as a string. Returns null if not found.
     * @param {CBaseEntity} entity
     * @param {property} property_name
     * @returns {string|null}
     */
    function GetPropType(entity, property_name);

    /**
     * Reads a 3D vector-valued netprop. Returns empty vector if not found.
     * @param {CBaseEntity} entity
     * @param {vector_property} property_name
     * @returns {Vector}
     */
    function GetPropVector(entity, property_name);

    /**
     * Reads a 3D vector-valued netprop from an array. Returns empty vector if not found.
     * @param {CBaseEntity} entity
     * @param {vector_array_property} property_name
     * @param {integer} array_element
     * @returns {Vector}
     */
    function GetPropVectorArray(entity, property_name, array_element);

    /**
     * Fills in a passed table with all props of a specified type
     * @param {CBaseEntity} entity
     * @param {integer} prop_type (0: SendTable, 1: DataMap).
     * @param {table} result
     */
    function GetTable(entity, prop_type, result);

    /**
     * Checks if a netprop exists.
     * @param {CBaseEntity} entity
     * @param {property} property_name
     * @returns {bool}
     */
    function HasProp(entity, property_name);

    /**
     * Sets a netprop to the specified boolean.
     * @param {CBaseEntity} entity
     * @param {bool_property} property_name
     * @param {bool} value
     */
    function SetPropBool(entity, property_name, value);

    /**
     * Sets a netprop from an array to the specified boolean.
     * @param {CBaseEntity} entity
     * @param {bool_array_property} property_name
     * @param {bool} value
     * @param {integer} array_element
     */
    function SetPropBoolArray(entity, property_name, value, array_element);

    /**
     * Sets an EHANDLE-valued netprop to reference the specified entity.
     * @param {CBaseEntity} entity
     * @param {entity_property} property_name
     * @param {CBaseEntity|null} value
     */
    function SetPropEntity(entity, property_name, value);

    /**
     * Sets an EHANDLE-valued netprop from an array to reference the specified entity.
     * @param {CBaseEntity} entity
     * @param {entity_array_property} property_name
     * @param {CBaseEntity|null} value
     * @param {integer} array_element
     */
    function SetPropEntityArray(entity, property_name, value, array_element);

    /**
     * Sets a netprop to the specified float.
     * @param {CBaseEntity} entity
     * @param {float_property} property_name
     * @param {float} value
     */
    function SetPropFloat(entity, property_name, value);

    /**
     * Sets a netprop from an array to the specified float.
     * @param {CBaseEntity} entity
     * @param {float_array_property} property_name
     * @param {float} value
     * @param {integer} array_element
     */
    function SetPropFloatArray(entity, property_name, value, array_element);

    /**
     * Sets a netprop to the specified integer.
     *
     * **Warning**: Do not override `m_iTeamNum` netprops on players or Engineer buildings permanently.
     * @param {CBaseEntity} entity
     * @param {integer_property} property_name
     * @param {integer} value
     */
    function SetPropInt(entity, property_name, value);

    /**
     * Sets a netprop from an array to the specified integer.
     * @param {CBaseEntity} entity
     * @param {integer_array_property} property_name
     * @param {integer} value
     * @param {integer} array_element
     */
    function SetPropIntArray(entity, property_name, value, array_element);

    /**
     * Sets a netprop to the specified string.
     * @param {CBaseEntity} entity
     * @param {string_property} property_name
     * @param {string|null} value
     */
    function SetPropString(entity, property_name, value);

    /**
     * Sets a netprop from an array to the specified string.
     * @param {CBaseEntity} entity
     * @param {string_array_property} property_name
     * @param {string|null} value
     * @param {integer} array_element
     */
    function SetPropStringArray(entity, property_name, value, array_element);

    /**
     * Sets a netprop to the specified vector.
     * @param {CBaseEntity} entity
     * @param {vector_property} property_name
     * @param {Vector} value
     */
    function SetPropVector(entity, property_name, value);

    /**
     * Sets a netprop from an array to the specified vector.
     * @param {CBaseEntity} entity
     * @param {vector_array_property} property_name
     * @param {Vector} value
     * @param {integer} array_element
     */
    function SetPropVectorArray(entity, property_name, value, array_element);
}

// ============================================================
// CScriptEntityOutputs (Game Instance: EntityOutputs)
// ============================================================

/**
 * Allows reading and manipulation of entity output data.
 */
class CScriptEntityOutputs {
    /**
     * Adds a new output to the entity.
     * @param {CBaseEntity} entity
     * @param {string} output_name
     * @param {string} targetname
     * @param {output} input_name
     * @param {string|null} parameter
     * @param {float} delay
     * @param {integer} times_to_fire
     */
    function AddOutput(entity, output_name, targetname, input_name, parameter, delay, times_to_fire);

    /**
     * Returns the number of array elements.
     * @param {CBaseEntity} entity
     * @param {output} output_name
     * @returns {integer}
     */
    function GetNumElements(entity, output_name);

    /**
     * Fills the passed table with output information.
     * @param {CBaseEntity} entity
     * @param {output} output_name
     * @param {table} result
     * @param {integer} array_element
     */
    function GetOutputTable(entity, output_name, result, array_element);

    /**
     * Returns true if an action exists for the output.
     * @param {CBaseEntity} entity
     * @param {output} output_name
     * @returns {bool}
     */
    function HasAction(entity, output_name);

    /**
     * Returns true if the output exists.
     * @param {CBaseEntity} entity
     * @param {output} output_name
     * @returns {bool}
     */
    function HasOutput(entity, output_name);

    /**
     * Removes an output from the entity.
     * @param {CBaseEntity} entity
     * @param {string} output_name
     * @param {string} targetname
     * @param {output} input_name
     * @param {string|null} parameter
     */
    function RemoveOutput(entity, output_name, targetname, input_name, parameter);
}

// ============================================================
// CScriptKeyValues
// ============================================================

/**
 * Script handle representation of a model's $keyvalues block.
 */
class CScriptKeyValues {
    /**
     * Find a sub key by the key name.
     * @param {string} key
     * @returns {CScriptKeyValues|null}
     */
    function FindKey(key);

    /**
     * Return the first sub key object.
     * @returns {CScriptKeyValues|null}
     */
    function GetFirstSubKey();

    /**
     * Return the key value as a bool.
     * @param {string} key
     * @returns {bool}
     */
    function GetKeyBool(key);

    /**
     * Return the key value as a float.
     * @param {string} key
     * @returns {float}
     */
    function GetKeyFloat(key);

    /**
     * Return the key value as an integer.
     * @param {string} key
     * @returns {integer}
     */
    function GetKeyInt(key);

    /**
     * Return the key value as a string.
     * @param {string} key
     * @returns {string}
     */
    function GetKeyString(key);

    /**
     * Return the next neighbor key object.
     * @returns {CScriptKeyValues|null}
     */
    function GetNextKey();

    /**
     * Returns true if the named key has no value.
     * @param {string} key
     * @returns {bool}
     */
    function IsKeyEmpty(key);

    /**
     * Whether the handle belongs to a valid key.
     * @returns {bool}
     */
    function IsValid();

    /**
     * Releases the contents of the instance.
     */
    function ReleaseKeyValues();
}

// ============================================================
// CPlayerVoiceListener (Game Instance: PlayerVoiceListener)
// ============================================================

/**
 * Tracks if any player is using voice and for how long.
 */
class CPlayerVoiceListener {
    /**
     * Returns the number of seconds the player has been continuously speaking.
     * @param {integer} player_index
     * @returns {float}
     */
    function GetPlayerSpeechDuration(player_index);

    /**
     * Returns whether the player specified is speaking.
     * @param {integer} player_index
     * @returns {bool}
     */
    function IsPlayerSpeaking(player_index);
}

// ============================================================
// CEnvEntityMaker extends CBaseEntity
// ============================================================

/**
 * Script handle class for env_entity_maker.
 */
class CEnvEntityMaker extends CBaseEntity {
    /**
     * Create an entity at the location of the maker.
     */
    function SpawnEntity();

    /**
     * Create an entity at the location of a specified entity instance.
     * @param {CBaseEntity} entity
     */
    function SpawnEntityAtEntityOrigin(entity);

    /**
     * Create an entity at a specified location and orientation.
     * @param {Vector} origin
     * @param {Vector} orientation Euler angle in degrees (pitch, yaw, roll)
     */
    function SpawnEntityAtLocation(origin, orientation);

    /**
     * Create an entity at the location of a named entity.
     * @param {string} targetname
     */
    function SpawnEntityAtNamedEntityOrigin(targetname);
}

// ============================================================
// CFuncTrackTrain extends CBaseEntity
// ============================================================

/**
 * Script handle class for func_tracktrain.
 */
class CFuncTrackTrain extends CBaseEntity {
    /**
     * Get a position on the track X seconds in the future.
     * @param {float} x
     * @param {float} speed
     * @returns {Vector}
     */
    function GetFuturePosition(x, speed);
}

// ============================================================
// CSceneEntity extends CBaseEntity
// ============================================================

/**
 * Script handle class for scripted_scene (VCD data).
 */
class CSceneEntity extends CBaseEntity {
    /**
     * Adds a team (by index) to the broadcast list.
     * @param {integer} index
     */
    function AddBroadcastTeamTarget(index);

    /**
     * Returns length of this scene in seconds.
     * @returns {float}
     */
    function EstimateLength();

    /**
     * Given an entity reference such as !target, get actual entity from scene object.
     * @param {string} reference
     * @returns {CBaseEntity|null}
     */
    function FindNamedEntity(reference);

    /**
     * If this scene is currently paused.
     * @returns {bool}
     */
    function IsPaused();

    /**
     * If this scene is currently playing.
     * @returns {bool}
     */
    function IsPlayingBack();

    /**
     * Given a dummy scene name and a vcd string, load the scene.
     * @param {string} scene_name
     * @param {string} scene
     * @returns {bool}
     */
    function LoadSceneFromString(scene_name, scene);

    /**
     * Removes a team (by index) from the broadcast list.
     * @param {integer} index
     */
    function RemoveBroadcastTeamTarget(index);
}

class CCallChainer {
    /**
     * Contains names of unprefixed functions, each with an array of functions to call.
     * @type {table}
     */
    chains = null

    /**
     * Prefix that functions should have to be added into the chains table. Set by the constructor.
     * @type {string}
     */
    prefix = null

    /**
     * If set, seek functions in this scope instead. Set by the constructor.
     * @type {table|null}
     */
    scope = null

    /**
     * Creates a CCallChainer object that'll collect functions that have a matching prefix in the given scope.
     * @param {string} function_prefix
     * @param {table|null} scope
     */
    constructor(function_prefix, scope = null);

    /**
     * Search for all non-native functions with matching prefixes, then push them into the chains table.
     */
    function PostScriptExecute();

    /**
     * Find an unprefixed function name in the chains table and call it with the given arguments.
     * @param {string} event
     * @varargs {any}
     * @returns {bool}
     */
    function Call(event, ...);
}

class CSimpleCallChainer {
    /**
     * All functions to be called by the Call() method.
     * @type {array}
     */
    chains = null

    /**
     * If set, names of non-native functions and prefix must be an exact match. Set by the constructor.
     * @type {bool}
     */
    exact_match = null

    /**
     * Prefix that functions should have to be added into the chain array. Set by the constructor.
     * @type {string}
     */
    prefix = null

    /**
     * If set, seek functions in this scope instead. Set by the constructor.
     * @type {table|null}
     */
    scope = null

    /**
     * Creates a CSimpleCallChainer object that'll collect functions that have a matching prefix in the given scope, unless it seek for an exact name match.
     * @param {string} function_prefix
     * @param {table|null} scope
     * @param {bool} exactMatch
     */
    constructor(function_prefix, scope = null, exactMatch = false);

    /**
     * Begin searching for all non-native functions with matching prefixes, then push them into the chain array.
     */
    function PostScriptExecute();

    /**
     * Call all functions inside the chain array with the given arguments.
     * @varargs {any}
     * @returns {bool}
     */
    function Call(...);
}

// ============================================================
// NextBotCombatCharacter extends CBaseCombatCharacter
// ============================================================

/**
 * Script handle class for non-playable combat characters operating under the NextBot system.
 */
class NextBotCombatCharacter extends CBaseCombatCharacter {
    /**
     * Clear immobile status.
     */
    function ClearImmobileStatus();

    /**
     * Flag this bot for update.
     * Tip: Use in think function to update nextbots faster than nb_update_frequency.
     * @param {bool} toggle
     */
    function FlagForUpdate(toggle);

    /**
     * Get this bot's body interface.
     * @returns {INextBotComponent}
     */
    function GetBodyInterface();

    /**
     * Get this bot's id.
     * @returns {integer}
     */
    function GetBotId();

    /**
     * How long have we been immobile.
     * @returns {float}
     */
    function GetImmobileDuration();

    /**
     * Return units/second below which this actor is considered immobile.
     * @returns {float}
     */
    function GetImmobileSpeedThreshold();

    /**
     * Get this bot's intention interface.
     * @returns {INextBotComponent}
     */
    function GetIntentionInterface();

    /**
     * Get this bot's locomotion interface.
     * @returns {ILocomotion}
     */
    function GetLocomotionInterface();

    /**
     * Get last update tick.
     * @returns {integer}
     */
    function GetTickLastUpdate();

    /**
     * Get this bot's vision interface.
     * @returns {INextBotComponent}
     */
    function GetVisionInterface();

    /**
     * Return true if given entity is our enemy.
     * @param {CBaseEntity} entity
     * @returns {bool}
     */
    function IsEnemy(entity);

    /**
     * Is this bot flagged for update.
     * @returns {bool}
     */
    function IsFlaggedForUpdate();

    /**
     * Return true if given entity is our friend.
     * @param {CBaseEntity} entity
     * @returns {bool}
     */
    function IsFriend(entity);

    /**
     * Return true if we haven't moved in awhile.
     * @returns {bool}
     */
    function IsImmobile();
}

// ============================================================
// INextBotComponent
// ============================================================

/**
 * Base script handle class for any interfaces belonging to a NextBotCombatCharacter entity.
 */
class INextBotComponent {
    /**
     * Recomputes the component update interval.
     * @returns {bool}
     */
    function ComputeUpdateInterval();

    /**
     * Returns the component update interval.
     * @returns {float}
     */
    function GetUpdateInterval();

    /**
     * Resets the internal update state.
     */
    function Reset();
}

// ============================================================
// ILocomotion extends INextBotComponent
// ============================================================

/**
 * The interface for interacting with a specific NextBot's movement brain.
 */
class ILocomotion extends INextBotComponent {
    /**
     * The primary locomotive method. Move towards goal position.
     * Tip: Put in a think function to make the entity move smoothly.
     * @param {Vector} goal
     * @param {float} goal_weight
     */
    function Approach(goal, goal_weight);

    /**
     * Reset stuck status to un-stuck.
     * @param {string} reason
     */
    function ClearStuckStatus(reason);

    /**
     * Initiate a jump to an adjacent high ledge. Returns false if climb can't start.
     * @param {Vector} goal_pos
     * @param {Vector} goal_forward
     * @param {CBaseEntity} obstacle
     * @returns {bool}
     */
    function ClimbUpToLedge(goal_pos, goal_forward, obstacle);

    /**
     * Returns false if no time has elapsed.
     * @returns {bool}
     */
    function ComputeUpdateInterval();

    /**
     * Move the bot to the precise given position immediately, updating internal state.
     * @param {Vector} pos
     */
    function DriveTo(pos);

    /**
     * Rotate body to face towards target.
     * Tip: Put in a think function for smooth rotation.
     * @param {Vector} target
     */
    function FaceTowards(target);

    /**
     * If the locomotor cannot jump over the gap, returns the fraction of the jumpable ray.
     * @param {Vector} from
     * @param {Vector} to
     * @returns {float}
     */
    function FractionPotentialGap(from, to);

    /**
     * If the locomotor could not move along the line given, returns the fraction of the walkable ray.
     * @param {Vector} from
     * @param {Vector} to
     * @param {bool} immediately
     * @returns {float}
     */
    function FractionPotentiallyTraversable(from, to, immediately);

    /**
     * Distance at which we will die if we fall.
     * @returns {float}
     */
    function GetDeathDropHeight();

    /**
     * Get desired speed for locomotor movement.
     * @returns {float}
     */
    function GetDesiredSpeed();

    /**
     * Return position of feet - the driving point where the bot contacts the ground.
     * @returns {Vector}
     */
    function GetFeet();

    /**
     * Return the current ground entity or null if not on the ground.
     * @returns {CBaseEntity|null}
     */
    function GetGround();

    /**
     * Return unit vector in XY plane describing direction of motion.
     * @returns {Vector}
     */
    function GetGroundMotionVector();

    /**
     * Surface normal of the ground we are in contact with.
     * @returns {Vector}
     */
    function GetGroundNormal();

    /**
     * Return current world space speed in XY plane.
     * @returns {float}
     */
    function GetGroundSpeed();

    /**
     * Return maximum acceleration of locomotor.
     * @returns {float}
     */
    function GetMaxAcceleration();

    /**
     * Return maximum deceleration of locomotor.
     * @returns {float}
     */
    function GetMaxDeceleration();

    /**
     * Return maximum height of a jump.
     * @returns {float}
     */
    function GetMaxJumpHeight();

    /**
     * Return unit vector describing our direction of motion.
     * @returns {Vector}
     */
    function GetMotionVector();

    /**
     * Get maximum running speed.
     * @returns {float}
     */
    function GetRunSpeed();

    /**
     * Return current world space speed (magnitude of velocity).
     * @returns {float}
     */
    function GetSpeed();

    /**
     * Get maximum speed bot can reach, regardless of desired speed.
     * @returns {float}
     */
    function GetSpeedLimit();

    /**
     * If delta Z is lower than this, we can step up the surface; otherwise we have to jump.
     * @returns {float}
     */
    function GetStepHeight();

    /**
     * Return how long we've been stuck.
     * @returns {float}
     */
    function GetStuckDuration();

    /**
     * Return Z component of unit normal of steepest traversable slope.
     * @returns {float}
     */
    function GetTraversableSlopeLimit();

    /**
     * Returns time between updates.
     * @returns {float}
     */
    function GetUpdateInterval();

    /**
     * Return current world space velocity.
     * @returns {Vector}
     */
    function GetVelocity();

    /**
     * Get maximum walking speed.
     * @returns {float}
     */
    function GetWalkSpeed();

    /**
     * Checks if there is a possible gap that will need to be jumped over.
     * @param {Vector} from
     * @param {Vector} to
     * @returns {float}
     */
    function HasPotentialGap(from, to);

    /**
     * Return true if this bot can climb arbitrary geometry it encounters.
     * @returns {bool}
     */
    function IsAbleToClimb();

    /**
     * Return true if this bot can jump across gaps in its path.
     * @returns {bool}
     */
    function IsAbleToJumpAcrossGaps();

    /**
     * Return true if given area can be used for navigation.
     * @param {CBaseEntity} area
     * @returns {bool}
     */
    function IsAreaTraversable(area);

    /**
     * Return true if we have tried to Approach() or DriveTo() very recently.
     * @returns {bool}
     */
    function IsAttemptingToMove();

    /**
     * Is jumping in any form.
     * @returns {bool}
     */
    function IsClimbingOrJumping();

    /**
     * Is climbing up to a high ledge.
     * @returns {bool}
     */
    function IsClimbingUpToLedge();

    /**
     * Return true if the entity handle is traversable.
     * @param {CBaseEntity} entity
     * @param {bool} immediately
     * @returns {bool}
     */
    function IsEntityTraversable(entity, immediately);

    /**
     * Return true if there is a gap at this position.
     * @param {Vector} pos
     * @param {Vector} forward
     * @returns {bool}
     */
    function IsGap(pos, forward);

    /**
     * Is jumping across a gap to the far side.
     * @returns {bool}
     */
    function IsJumpingAcrossGap();

    /**
     * Return true if standing on something.
     * @returns {bool}
     */
    function IsOnGround();

    /**
     * Checks if this locomotor could potentially move along the line given.
     * @param {Vector} from
     * @param {Vector} to
     * @param {bool} immediately
     * @returns {float}
     */
    function IsPotentiallyTraversable(from, to, immediately);

    /**
     * Is running?
     * @returns {bool}
     */
    function IsRunning();

    /**
     * Is in the middle of a complex action that shouldn't be interrupted.
     * @returns {bool}
     */
    function IsScrambling();

    /**
     * Return true if bot is stuck.
     * @returns {bool}
     */
    function IsStuck();

    /**
     * Initiate a simple undirected jump in the air.
     */
    function Jump();

    /**
     * Initiate a jump across an empty volume of space to far side.
     * @param {Vector} goal_pos
     * @param {Vector} goal_forward
     */
    function JumpAcrossGap(goal_pos, goal_forward);

    /**
     * Manually run the OnLandOnGround callback.
     * @param {CBaseEntity} ground
     */
    function OnLandOnGround(ground);

    /**
     * Manually run the OnLeaveGround callback.
     * @param {CBaseEntity} ground
     */
    function OnLeaveGround(ground);

    /**
     * Resets motion, stuck state etc.
     */
    function Reset();

    /**
     * Set desired movement speed to running.
     */
    function Run();

    /**
     * Set desired speed for locomotor movement.
     * @param {float} speed
     */
    function SetDesiredSpeed(speed);

    /**
     * Set maximum speed bot can reach, regardless of desired speed.
     * @param {float} limit
     */
    function SetSpeedLimit(limit);

    /**
     * Set desired movement speed to stopped.
     */
    function Stop();

    /**
     * Set desired movement speed to walking.
     */
    function Walk();
}

// ============================================================
// Vector (Data Type)
// ============================================================

/**
 * Squirrel equivalent of the C++ Vector class.
 * A three-dimensional vector with overloaded arithmetic operations for both Vectors and scalar values.
 */
class Vector {
    /**
     * Cartesian X axis.
     * @type {float}
     */
    x = null

    /**
     * Cartesian Y axis.
     * @type {float}
     */
    y = null

    /**
     * Cartesian Z axis.
     * @type {float}
     */
    z = null

    /**
     * Creates a new vector with the specified Cartesian coordinates.
     * @param {float} x
     * @param {float} y
     * @param {float} z
     */
    constructor(x = 0.0, y = 0.0, z = 0.0);

    /**
     * Returns the sum of both classes's members.
     * @param {Vector|QAngle} other
     * @returns {Vector}
     */
    function _add(other);

    /**
     * Returns the subtraction of both classes's members.
     * @param {Vector|QAngle} other
     * @returns {Vector}
     */
    function _sub(other);

    /**
     * Returns the multiplication of a Vector against a scalar.
     * @param {float} other
     * @returns {Vector}
     */
    function _mul(other);

    /**
     * The vector product of two vectors. Returns a vector orthogonal to the input vectors.
     * @param {Vector} factor
     * @returns {Vector}
     */
    function Cross(factor);

    /**
     * The scalar product of two vectors.
     * @param {Vector} factor
     * @returns {float}
     */
    function Dot(factor);

    /**
     * Magnitude of the vector.
     * @returns {float}
     */
    function Length();

    /**
     * The magnitude of the vector squared.
     * @returns {float}
     */
    function LengthSqr();

    /**
     * Returns the magnitude of the vector on the x-y plane.
     * @returns {float}
     */
    function Length2D();

    /**
     * Returns the square of the magnitude of the vector on the x-y plane.
     * @returns {float}
     */
    function Length2DSqr();

    /**
     * Normalizes the vector in place and returns its length.
     * @returns {float}
     */
    function Norm();

    /**
     * Scales the vector magnitude.
     * @param {float} factor
     * @returns {Vector}
     */
    function Scale(factor);

    /**
     * Returns a string without separating commas.
     * @returns {string}
     */
    function ToKVString();

    /**
     * Returns a human-readable string.
     * @returns {string}
     */
    function tostring();
}

// ============================================================
// QAngle (Data Type)
// ============================================================

/**
 * Squirrel equivalent of the C++ QAngle class.
 * Represents a three-dimensional orientation as Euler angles.
 */
class QAngle {
    /**
     * Pitch in degrees.
     * @type {float}
     */
    x = null

    /**
     * Yaw in degrees.
     * @type {float}
     */
    y = null

    /**
     * Roll in degrees.
     * @type {float}
     */
    z = null

    /**
     * Creates a new QAngle.
     * @param {float} pitch
     * @param {float} yaw
     * @param {float} roll
     */
    constructor(pitch = 0.0, yaw = 0.0, roll = 0.0);

    /**
     * Returns the sum of both classes's members.
     * @param {QAngle|Vector} other
     * @returns {QAngle}
     */
    function _add(other);

    /**
     * Returns the subtraction of both classes's members.
     * @param {QAngle|Vector} other
     * @returns {QAngle}
     */
    function _sub(other);

    /**
     * QAngle multiplied by a number.
     * @param {float} other
     * @returns {QAngle}
     */
    function _mul(other);

    /**
     * @param {string|null} start
     * @returns {float}
     */
    function _nexti(start);

    /**
     * Returns the Forward Vector of the angles.
     * @returns {Vector}
     */
    function Forward();

    /**
     * Returns the right Vector of the angles.
     *
     * **Note**: Despite being named "Left", this actually returns the right vector.
     * @returns {Vector}
     */
    function Left();

    /**
     * Returns the pitch angle in degrees.
     * @returns {float}
     */
    function Pitch();

    /**
     * Returns the roll angle in degrees.
     * @returns {float}
     */
    function Roll();

    /**
     * Returns a string with the values separated by one space.
     * @returns {string}
     */
    function ToKVString();

    /**
     * Returns a quaternion representation of the orientation.
     * @returns {Quaternion}
     */
    function ToQuat();

    /**
     * Returns the Up Vector of the angles.
     * @returns {Vector}
     */
    function Up();

    /**
     * Returns the yaw angle in degrees.
     * @returns {float}
     */
    function Yaw();
}

class Vector2D {
    /** @type {float} */
    x = null

    /** @type {float} */
    y = null

    /**
     * Creates a new 2-dimensional vector with the specified Cartesian coordiantes.
     * @param {float} x
     * @param {float} y
     */
    constructor(x = 0.0, y = 0.0);

    /**
     * Returns the sum of both classes's members.
     * @param {Vector2D} other
     * @returns {Vector2D}
     */
    function _add(other);

    /**
     * Returns the subtraction of both classes's members.
     * @param {Vector2D} other
     * @returns {Vector2D}
     */
    function _sub(other);

    /**
     * Returns the multiplication of a Vector against a scalar.
     * @param {float} other
     * @returns {Vector2D}
     */
    function _mul(other);


    /**
     * The scalar product of two vectors.
     * @param {Vector2D} factor
     * @returns {float}
     */
    function Dot(factor);

    /**
     * Magnitude of the vector.
     * @returns {float}
     */
    function Length();

    /**
     * The magnitude of the vector squared.
     * @returns {float}
     */
    function LengthSqr();

    /**
     * Normalizes the vector in place and returns its length.
     * @returns {float}
     */
    function Norm();

    /**
     * Returns a string without separating commas.
     * @returns {string}
     */
    function ToKVString();
}

class Vector4D {
    /** @type {float} */
    x = null

    /** @type {float} */
    y = null

    /** @type {float} */
    z = null

    /** @type {float} */
    w = null

    /**
     * Creates a new 2-dimensional vector with the specified Cartesian coordiantes.
     * @param {float} x
     * @param {float} y
     * @param {float} z
     * @param {float} w
     */
    constructor(x = 0.0, y = 0.0, z = 0.0, w = 0.0);

    /**
     * Returns the sum of both classes's members.
     * @param {Vector4D} other
     * @returns {Vector4D}
     */
    function _add(other);

    /**
     * Returns the subtraction of both classes's members.
     * @param {Vector4D} other
     * @returns {Vector4D}
     */
    function _sub(other);

    /**
     * Returns the multiplication of a Vector against a scalar.
     * @param {float} other
     * @returns {Vector4D}
     */
    function _mul(other);

    /**
     * The scalar product of two vectors.
     * @param {Vector4D} factor
     * @returns {float}
     */
    function Dot(factor);

    /**
     * Magnitude of the vector.
     * @returns {float}
     */
    function Length();

    /**
     * The magnitude of the vector squared.
     * @returns {float}
     */
    function LengthSqr();

    /**
     * Normalizes the vector in place and returns its length.
     * @returns {float}
     */
    function Norm();

    /**
     * Returns a string without separating commas.
     * @returns {string}
     */
    function ToKVString();
}

// ============================================================
// Quaternion (Data Type)
// ============================================================

/**
 * Quaternion represents rotations in three-dimensional space.
 */
class Quaternion {
    /**
     * Vector component along the i axis.
     * @type {float}
     */
    x = null

    /**
     * Vector component along the j axis.
     * @type {float}
     */
    y = null

    /**
     * Vector component along the k axis.
     * @type {float}
     */
    z = null

    /**
     * Scalar part.
     * @type {float}
     */
    w = null

    /**
     * No parameters: creates a new identity quaternion (0, 0, 0, 1).
     * Otherwise: creates a new quaternion of the form w + xi + yj + zk.
     * @param {float} x
     * @param {float} y
     * @param {float} z
     * @param {float} w
     */
    constructor(x = 0.0, y = 0.0, z = 0.0, w = 0.0);

    /**
     * @param {Quaternion} other
     * @returns {Quaternion}
     */
    function _add(other);

    /**
     * @param {Quaternion} other
     * @returns {Quaternion}
     */
    function _sub(other);

    /**
     * @param {float} other
     * @returns {Quaternion}
     */
    function _mul(other);

    /**
     * The 4D scalar product of two quaternions.
     * @param {Quaternion} factor
     * @returns {float}
     */
    function Dot(factor);

    /**
     * Returns a quaternion with the complementary rotation.
     * @returns {Quaternion}
     */
    function Invert();

    /**
     * Normalizes the quaternion.
     * @returns {float}
     */
    function Norm();

    /**
     * Recomputes the quaternion from the supplied Euler angles.
     * @param {float} pitch
     * @param {float} yaw
     * @param {float} roll
     */
    function SetPitchYawRoll(pitch, yaw, roll);

    /**
     * Returns a string with the values separated by one space.
     * @returns {string}
     */
    function ToKVString();

    /**
     * Returns the angles resulting from the rotation.
     * @returns {QAngle}
     */
    function ToQAngle();
}

// ============================================================
// GLOBAL FUNCTIONS - Shared
// ============================================================

/**
 * Sets a function in the entity's script to rerun by itself constantly.
 * Pass null as the function name to remove a think function.
 * The default think interval is 0.1s, unless overridden by returning a different time interval in seconds.
 * TF2 runs at 66 ticks per second, so the lowest possible interval is 0.015 seconds.
 * Return -1 to think every tick.
 * The highest interval where all clients will interpolate entities is 0.05 (20 times per second).
 * @param {CBaseEntity} entity
 * @param {string|null} function_name
 */
function AddThinkToEnt(entity, function_name);

/**
 * Test value and if not true, throws exception, optionally with message.
 * @param {bool} value
 * @throws {string|null}
 * @param {string|null} optional_message
 */
function Assert(value, optional_message = null);

/**
 * Empties the tables of game event callback functions.
 * @deprecated Do NOT use this! It removes all events including those from other scripts.
 */
function ClearGameEventCallbacks();

/**
 * Create a prop.
 * @param {classname} classname
 * @param {Vector} origin
 * @param {string} model_name
 * @param {integer} activity
 * @returns {CBaseAnimating|null}
 */
function CreateProp(classname, origin, model_name, activity);

/**
 * Create a scene entity to play the specified scene. Can only be created during map initialization.
 * @param {string} scene
 * @returns {CBaseAnimating|null}
 */
function CreateSceneEntity(scene);

/**
 * The current level of the developer console variable.
 * @returns {integer}
 */
function developer();

/**
 * Dispatches a one-off particle system.
 *
 * **Warning**: Does NOT work if called from a player think or `OnTakeDamage` caused by hitscan/melee.
 * @param {string} name
 * @param {Vector} origin
 * @param {Vector} direction
 */
function DispatchParticleEffect(name, origin, direction);

/**
 * @param {any} symbol_or_table
 * @param {any} item_if_symbol
 * @param {string|null} description_if_symbol
 */
function Document(symbol_or_table, item_if_symbol = null, description_if_symbol = null)

/**
 * Generate an entity I/O event.
 * @param {string} target
 * @param {string} action
 * @param {string|null} value
 * @param {float} delay
 * @param {CBaseEntity|null} activator
 * @param {CBaseEntity|null} caller
 */
function DoEntFire(target, action, value, delay, activator, caller);

/**
 * Used internally by IncludeScript
 * @param {script} file
 * @param {table|class|instance|null} scope
 * @returns {bool}
 * @hide
 */
function DoIncludeScript(file, scope);

/**
 * Execute a script and put all its content for the argument passed to the scope parameter.
 * The file must have the .nut extension.
 * @param {script} file
 * @param {table|class|instance|null} scope
 * @returns {bool}
 */
function IncludeScript(file, scope = null);

/**
 * Play named sound on an entity using configurations similar to ambient_generic.
 * @param {string} sound_name
 * @param {float} volume
 * @param {integer} soundlevel
 * @param {integer} pitch
 * @param {CBaseEntity} entity
 */
function EmitAmbientSoundOn(sound_name, volume, soundlevel, pitch, entity);

/**
 * Stop named sound on an entity using configurations similar to ambient_generic.
 * @param {string} sound_name
 * @param {CBaseEntity} entity
 */
function StopAmbientSoundOn(sound_name, entity);

/**
 * Play a sound with extended parameters.
 *
 * See the [EmitSoundEx](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/EmitSoundEx) for more details.
 * @param {table} params
 * ## Keys:
 * ```sqDoc
 * sound_name: string
 * channel?: integer
 * volume?: float
 * sound_level?: integer
 * flags?: integer
 * pitch?: integer
 * special_dsp?: integer
 * origin?: Vector
 * delay?: float,
 * sound_time?: float
 * entity?: CBaseEntity|null
 * speaker_entity?: CBaseEntity|null
 * filter_type?: integer
 * filter_param?: integer
 * ```
 */
function EmitSoundEx(params);

/**
 * Play named sound on given entity. The sound must be precached first.
 *
 * **Warning**: Looping sounds will not stop on the entity when it's destroyed.
 * @param {string} sound_script
 * @param {CBaseEntity} entity
 */
function EmitSoundOn(sound_script, entity);

/**
 * Stop named sound on an entity.
 * @param {string} sound_script
 * @param {CBaseEntity} entity
 */
function StopSoundOn(sound_script, entity);

/**
 * Play named sound only on the client for the specified player.
 *
 * **Note**: Only supports soundscripts.
 * @param {string} sound_script
 * @param {CBaseEntity} player
 */
function EmitSoundOnClient(sound_script, player);

/**
 * Wrapper for DoEntFire() that sets activator to null. Negative delays are clamped to 0.
 * @param {string} target
 * @param {input} action
 * @param {string|null} value
 * @param {float} delay
 * @param {CBaseEntity|null} activator
 */
function EntFire(target, action, value = null, delay = 0.0, activator = null);

/**
 * Generate an entity I/O event by handle. Negative delays are clamped to 0.
 *
 * **Note**: With 0 delay, processed at end of frame. Use AcceptInput for instant/synchronous I/O.
 * @param {CBaseEntity} entity
 * @param {input} action
 * @param {string|null} value
 * @param {float} delay
 * @param {CBaseEntity|null} activator
 * @param {CBaseEntity|null} caller
 */
function EntFireByHandle(entity, action, value, delay, activator, caller);


/**
 * Turn an entity index integer to an HScript representing that entity's script instance.
 * @param {integer} entindex
 * @returns {CBaseEntity|null}
 */
function EntIndexToHScript(entindex);

/**
 * Reads a string from file located in the game's scriptdata folder.
 * Returns the string from the file, null if no file or file is greater than 16384 bytes.
 * @param {string} file
 * @returns {string|null}
 */
function FileToString(file);

/**
 * Fire a game event to a listening callback function in script.
 *
 * **Note**: Does not fire an event that the game will pick up. Use SendGlobalGameEvent for real events.
 * @param {string} name
 * @param {table} params
 * @returns {bool}
 */
function FireGameEvent(name, params);

/**
 * Fire a script hook to a listening callback function in script.
 * @param {string} name
 * @param {table} params
 * @returns {bool}
 */
function FireScriptHook(name, params);

/**
 * Get the time spent on the server in the last frame. Usually 0.015 (default tickrate).
 * @returns {float}
 */
function FrameTime();

/**
 * Gets the level of 'developer'.
 * @returns {integer}
 */
function GetDeveloperLevel();

/**
 * Returns the engines current frame count.
 * @returns {integer}
 */
function GetFrameCount();

/**
 * Returns a string that describes the passed in function's signature.
 * @param {function} func
 * @param {string} prefix
 * @returns {string|null}
 */
function GetFunctionSignature(func, prefix);

/**
 * Get the local player on a listen server. Returns null on dedicated servers.
 * @returns {CTFPlayer|null}
 */
function GetListenServerHost();

/**
 * Get the name of the map without extension.
 * @returns {string}
 */
function GetMapName();

/**
 * Returns the index of the named model. Returns -1 if not loaded.
 * @param {string} model_name
 * @returns {integer}
 */
function GetModelIndex(model_name);

/**
 * Returns the angular velocity of the entity
 * @param {CBaseEntity} entity
 * @deprecated Use the `GetPhysAngularVelocity` method on the entity instead.
 */
function GetPhysAngularVelocity(entity);

/**
 * Returns the velocity of the entity
 * @param {CBaseEntity} entity
 * @deprecated Use the `GetPhysVelocity` method on the entity instead.
 */
function GetPhysVelocity(entity);

/**
 * Given a user id, return the entity, or null.
 * @param {integer} userid
 * @returns {CTFPlayer|null}
 */
function GetPlayerFromUserID(userid);

/**
 * Returns float duration of the sound.
 *
 * **Warning**: Does not work on dedicated servers.
 * @param {string} sound_name
 * @param {string|null} actor_model_name
 * @returns {float}
 */
function GetSoundDuration(sound_name, actor_model_name);

/**
 * Returns true if this server is a dedicated server.
 * @returns {bool}
 */
function IsDedicatedServer();

/**
 * Checks if the model_name is precached.
 * @param {string} model_name
 * @returns {bool}
 */
function IsModelPrecached(model_name);

/**
 * Checks if the sound_name is precached.
 * @param {string} sound_name
 * @returns {bool}
 */
function IsSoundPrecached(sound_name);

/**
 * Is this player/entity a puppet or AI bot.
 * @param {CTFPlayer} player
 * @returns {bool}
 */
function IsPlayerABot(player);

/**
 * Fills out a table with the local time.
 *
 * **Warning**: The month will be 1-12 rather than 0-11.
 * @param {table} result
 */
function LocalTime(result);

/**
 * Get the current number of max clients set by the maxplayers command.
 * @returns {float}
 */
function MaxClients();

/**
 * Get a script handle of a player using the player index.
 * @param {integer} index
 * @returns {CTFPlayer|null}
 */
function PlayerInstanceFromIndex(index);

/**
 * Precache an entity from KeyValues in a table.
 * @param {table} keyvalues
 * @returns {bool}
 */
function PrecacheEntityFromTable(keyvalues);

/**
 * Precache a studio model or sprite model and return model index.
 * @param {string} model_name
 * @returns {integer}
 */
function PrecacheModel(model_name);

/**
 * Precache a soundscript or raw WAV/MP3 sound.
 * @param {string} sound_name
 * @returns {bool}
 */
function PrecacheScriptSound(sound_name);

/**
 * Precache a raw WAV/MP3 sound.
 * @param {string} sound_name
 * @returns {bool}
 */
function PrecacheSound(sound_name);

/**
 * Generate a random floating-point number within a range, inclusive.
 * @param {float} min
 * @param {float} max
 * @returns {float}
 */
function RandomFloat(min, max);

/**
 * Generate a random integer within a range, inclusive.
 * @param {integer} min
 * @param {integer} max
 * @returns {integer}
 */
function RandomInt(min, max);

/**
 * Register as a listener for a game event from script.
 * @param {string} event_name
 */
function RegisterScriptGameEventListener(event_name);

/**
 * Register as a listener for a script hook from script.
 * @param {string} name
 */
function RegisterScriptHookListener(name);

/**
 * Rotate a QAngle by another QAngle.
 * @param {QAngle} initial
 * @param {QAngle} rotation
 * @returns {QAngle}
 */
function RotateOrientation(initial, rotation);

/**
 * Rotate the input Vector around an origin.
 * @param {Vector} origin
 * @param {QAngle} rotation
 * @param {Vector} input
 * @returns {Vector}
 */
function RotatePosition(origin, rotation, input);

/**
 * Start a customisable screenfade. If no player is specified, applies to all players.
 * @param {CTFPlayer} player
 * @param {integer} red
 * @param {integer} green
 * @param {integer} blue
 * @param {integer} alpha
 * @param {float} fade_time
 * @param {float} fade_hold
 * @param {integer} flags See [Constants.FFADE](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#FFADE)
 */
function ScreenFade(player, red, green, blue, alpha, fade_time, fade_hold, flags);

/**
 * Start a customisable screenshake.
 * @param {Vector} center
 * @param {float} amplitude
 * @param {float} frequency
 * @param {float} duration
 * @param {float} radius
 * @param {integer} command See [Constants.SHAKE_COMMAND](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#SHAKE_COMMAND) (0=start, 1=stop)
 * @param {bool} air_shake
 */
function ScreenShake(center, amplitude, frequency, duration, radius, command, air_shake);

/**
 * Returns whether script hooks are currently enabled.
 * @returns {bool}
 */
function ScriptHooksEnabled();

/**
 * Sends a real game event to everything.
 * @param {string} event_name
 * @param {table} params
 * @returns {bool}
 */
function SendGlobalGameEvent(event_name, params);

/**
 * Issues a command to the local client. Does nothing on dedicated servers.
 * @param {string} command
 */
function SendToConsole(command);

/**
 * Issues a command to the server, as if typed in the console.
 * @param {string} command
 */
function SendToServerConsole(command);

/**
 * Copy of SendToServerConsole with another name for compatibility.
 * @param {string} command
 */
function SendToConsoleServer(command);

/**
 * Sets a USERINFO client ConVar for a fakeclient.
 * @param {CTFBot} bot
 * @param {client_convar} cvar
 * @param {string} value
 */
function SetFakeClientConVarValue(bot, cvar, value);

/**
 * Sets the current skybox texture. The path is relative to "materials/skybox/".
 * @param {string} texture
 */
function SetSkyboxTexture(texture);

/**
 * Spawn entity from KeyValues in table.
 * @param {classname} name
 * @param {table} keyvalues
 * @returns {CBaseEntity|null}
 */
function SpawnEntityFromTable(name, keyvalues);

/**
 * Hierarchically spawn an entity group from a set of spawn tables.
 * @param {table} groups
 * @returns {bool}
 */
function SpawnEntityGroupFromTable(groups);

/**
 * Stores a string as a file, located in the game's scriptdata folder.
 *
 * **Warning**: Performance varies by hardware; only call at checkpoints.
 * @param {string} file
 * @param {string} content
 */
function StringToFile(file, content);

/**
 * Get the current time since map load in seconds.
 * @returns {float}
 */
function Time();

/**
 * Trace a ray. Return fraction along line that hits world or models.
 * @param {Vector} start
 * @param {Vector} end
 * @param {CBaseEntity|null} ignore
 * @returns {float}
 */
function TraceLine(start, end, ignore);

/**
 * Different version of `TraceLine` that also hits players and NPCs.
 * @param {Vector} start
 * @param {Vector} end
 * @param {CBaseEntity|null} ignore
 * @returns {float}
 */
function TraceLinePlayersIncluded(start, end, ignore);

/**
 * Extended version of `TraceLine`. The passed in table requires to have parameters and will be modified to contain new ones
 *
 * See [TraceLineEx](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/TraceLineEx) for more details
 *
 * **Warning**: Setting any input parameters which expect an instance to a primitive type will crash the server.
 * # Input table
 * ```sqDoc
 * start: Vector
 * end: Vector
 * mask: integer
 * ignore: CBaseEntity
 * ```
 * # Output table
 * ```sqDoc
 * pos: Vector
 * fraction: float
 * hit: bool
 * enthit?: CBaseEntity
 * startsolid?: bool
 * allsolid?: bool
 * startpos: Vector
 * endpos: Vector
 * plane_normal?: Vector
 * plane_dist?: float
 * surface_name?: string
 * surface_flags?: integer
 * surface_props?: integer
 * ```
 * @param {table} params
 * @returns {bool} `false` if the user didn't specify a valid `start` or `end`, `true` otherwise.
 *                 You don't need to check this return usually.
 */
function TraceLineEx(params);

/**
 * Trace a box (AABB). The passed in table requires to have parameters and will be modified to contain new ones
 *
 * See [TraceHull](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/TraceHull) for more details
 *
 * **Warning**: Setting any input parameters which expect an instance to a primitive type will crash the server.
 * # Input table
 * ```sqDoc
 * start: Vector
 * end: Vector
 * hullmin: Vector
 * hullmax: Vector
 * mask: integer
 * ignore: CBaseEntity
 * ```
 * # Output table
 * ```sqDoc
 * pos: Vector
 * fraction: float
 * hit: bool
 * enthit?: CBaseEntity
 * startsolid?: bool
 * allsolid?: bool
 * startpos: Vector
 * endpos: Vector
 * plane_normal?: Vector
 * plane_dist?: float
 * surface_name?: string
 * surface_flags?: integer
 * surface_props?: integer
 * ```
 * @param {table} params
 * @returns {bool} `false` if the user didn't specify a valid `start`, `end`, `hullmin` or `hullmax`, `true` otherwise.
 *                 You don't need to check this return usually.
 */
function TraceHull(params);

/**
 * Generate a string guaranteed to be unique across the life of the script VM.
 * @param {string} suffix
 * @returns {string}
 */
function UniqueString(suffix = "");

/**
 * Internal function called by `UniqueString`
 * @param {string|null} suffix
 * @returns {string}
 * @hide
 */
function DoUniqueString(suffix);

/**
 * Wrapper that registers callbacks for `OnGameEvent_x` and `OnScriptEvent_` functions.
 * @param {table} scope
 */
function __CollectGameEventCallbacks(scope);

// ============================================================
// GLOBAL FUNCTIONS - Team Fortress 2
// ============================================================

/**
 * @returns {bool}
 */
function AllowThirdPersonCamera();

/**
 * @returns {bool}
 */
function ArePlayersInHell();

/**
 * May a flag be captured?
 * @returns {bool}
 */
function FlagsMayBeCapped();

/**
 * Whether to force on MvM-styled upgrades on/off.
 * @param {integer} state 0=default, 1=force off, 2=force on
 */
function ForceEnableUpgrades(state);

/**
 * Forces payload pushing logic.
 * @param {integer} state 0=default, 1=force off, 2=force on.
 */
function ForceEscortPushLogic(state);

/**
 * Does the current gamemode have currency?
 * @returns {bool}
 */
function GameModeUsesCurrency();

/**
 * Does the current gamemode have minibosses?
 * @returns {bool}
 */
function GameModeUsesMiniBosses();

/**
 * Does the current gamemode have upgrades?
 * @returns {bool}
 */
function GameModeUsesUpgrades();

/**
 * Get class limit for class.
 * @param {integer} class_number See [Constants.ETFClass](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFClass)
 * @returns {integer}
 */
function GetClassLimit(class_number);

/**
 * @returns {float}
 */
function GetGravityMultiplier();

/**
 * @returns {bool}
 */
function GetMannVsMachineAlarmStatus();

/**
 * @returns {bool}
 */
function GetOvertimeAllowedForCTF();

/**
 * Get current round state.
 * @returns {integer} See [Constants.ERoundState](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ERoundState)
 */
function GetRoundState();

/**
 * Get the current stopwatch state.
 * @returns {integer} See [Constants.EStopwatchState](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#EStopwatchState)
 */
function GetStopWatchState();

/**
 * Who won!
 * @returns {integer}
 */
function GetWinningTeam();

/**
 * @returns {bool}
 */
function HaveStopWatchWinner();

/**
 * Are we in the pre-match/setup state?
 * @returns {bool}
 */
function InMatchStartCountdown();

/**
 * Currently in overtime?
 * @returns {bool}
 */
function InOvertime();

/**
 * @returns {bool}
 */
function IsAttackDefenseMode();

/**
 * Are we in birthday mode?
 * @returns {bool}
 */
function IsBirthday();

/**
 * Playing competitive?
 * @returns {bool}
 */
function IsCompetitiveMode();

/**
 * The absence of arena, mvm, tournament mode, etc.
 * @returns {bool}
 */
function IsDefaultGameMode();

/**
 * Is the given holiday active?
 * @param {integer} holiday See [Constants.EHoliday](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#EHoliday)
 * @returns {bool}
 */
function IsHolidayActive(holiday);

/**
 * Playing a holiday map?
 * @param {integer} holiday See [Constants.EHoliday](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#EHoliday)
 * @returns {bool}
 */
function IsHolidayMap(holiday);

/**
 * Playing arena mode?
 * @returns {bool}
 */
function IsInArenaMode();

/**
 * Playing king of the hill mode?
 * @returns {bool}
 */
function IsInKothMode();

/**
 * Playing medieval mode?
 * @returns {bool}
 */
function IsInMedievalMode();

/**
 * Are we waiting for some stragglers?
 * @returns {bool}
 */
function IsInWaitingForPlayers();

/**
 * Playing MvM?
 * @returns {bool}
 */
function IsMannVsMachineMode();

/**
 * Are players allowed to refund their upgrades?
 * @returns {bool}
 */
function IsMannVsMachineRespecEnabled();

/**
 * Playing casual?
 * @returns {bool}
 */
function IsMatchTypeCasual();

/**
 * Playing competitive?
 * @returns {bool}
 */
function IsMatchTypeCompetitive();

/**
 * No ball games.
 * @returns {bool}
 */
function IsPasstimeMode();

/**
 * Playing powerup mode?
 * @returns {bool}
 */
function IsPowerupMode();

/**
 * @returns {bool}
 */
function IsPVEModeActive();

/**
 * If an engineer places a building, will it immediately upgrade?
 * @returns {bool}
 */
function IsQuickBuildTime();

/**
 * @returns {bool}
 */
function IsTruceActive();

/**
 * @returns {bool}
 */
function IsUsingGrapplingHook();

/**
 * @returns {bool}
 */
function IsUsingSpells();

/**
 * @returns {bool}
 */
function MapHasMatchSummaryStage();

/**
 * @returns {bool}
 */
function MatchmakingShouldUseStopwatchMode();

/**
 * @param {integer} team See [Constants.ETFTeam](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#ETFTeam)
 * @returns {bool}
 */
function PlayerReadyStatus_ArePlayersOnTeamReady(team);

/**
 * @returns {bool}
 */
function PlayerReadyStatus_HaveMinPlayersToEnable();

/**
 */
function PlayerReadyStatus_ResetState();

/**
 * @returns {bool}
 */
function PlayersAreOnMatchSummaryStage();

/**
 * Are points able to be captured?
 * @returns {bool}
 */
function PointsMayBeCaptured();

/**
 * @param {float} multiplier
 */
function SetGravityMultiplier(multiplier);

/**
 * @param {bool} status
 */
function SetMannVsMachineAlarmStatus(status);

/**
 * @param {bool} state
 */
function SetOvertimeAllowedForCTF(state);

/**
 * @param {bool} state
 */
function SetPlayersInHell(state);

/**
 * @param {bool} state
 */
function SetUsingSpells(state);

/**
 * @returns {bool}
 */
function UsePlayerReadyStatusMode();

// ============================================================
// GLOBAL FUNCTIONS - Printing and Drawing
// ============================================================

/**
 * Print a client message. Pass null instead of a valid player to send to all clients.
 * When printing to chat (HUD_PRINTTALK), use \x07RRGGBB for custom colors.
 * @param {CTFPlayer|null} player
 * @param {integer} destination See [Constants.EHudNotify](https://developer.valvesoftware.com/wiki/Team_Fortress_2/Scripting/Script_Functions/Constants#EHudNotify)
 * @param {string} message
 */
function ClientPrint(player, destination, message);

/**
 * Draw a debug overlay box.
 *
 * **Warning**: Requires developer cvar to be enabled.
 * @param {Vector} origin
 * @param {Vector} min
 * @param {Vector} max
 * @param {integer} r
 * @param {integer} g
 * @param {integer} b
 * @param {integer} alpha
 * @param {float} duration
 */
function DebugDrawBox(origin, min, max, r, g, b, alpha, duration);

/**
 * Draw a debug oriented box.
 * @param {Vector} origin
 * @param {Vector} min
 * @param {Vector} max
 * @param {QAngle} direction
 * @param {Vector} rgb
 * @param {integer} alpha
 * @param {float} duration
 */
function DebugDrawBoxAngles(origin, min, max, direction, rgb, alpha, duration);

/**
 * Draw a debug forward box.
 * @param {Vector} center
 * @param {Vector} min
 * @param {Vector} max
 * @param {Vector} forward
 * @param {Vector} rgb
 * @param {float} alpha
 * @param {float} duration
 */
function DebugDrawBoxDirection(center, min, max, forward, rgb, alpha, duration);

/**
 * Draw a debug circle.
 * @param {Vector} center
 * @param {Vector} rgb
 * @param {float} alpha
 * @param {float} radius
 * @param {bool} ztest
 * @param {float} duration
 */
function DebugDrawCircle(center, rgb, alpha, radius, ztest, duration);

/**
 * Try to clear all the debug overlay info.
 */
function DebugDrawClear();

/**
 * Draw a debug overlay line.
 * @param {Vector} start
 * @param {Vector} end
 * @param {integer} red
 * @param {integer} green
 * @param {integer} blue
 * @param {bool} z_test
 * @param {float} time
 */
function DebugDrawLine(start, end, red, green, blue, z_test, time);

/**
 * Draw a debug line using color vec.
 * @param {Vector} start
 * @param {Vector} end
 * @param {Vector} rgb
 * @param {bool} ztest
 * @param {float} duration
 */
function DebugDrawLine_vCol(start, end, rgb, ztest, duration);

/**
 * Draw text with a line offset.
 * @param {float} x
 * @param {float} y
 * @param {integer} line_offset
 * @param {string} text
 * @param {integer} r
 * @param {integer} g
 * @param {integer} b
 * @param {integer} a
 * @param {float} duration
 */
function DebugDrawScreenTextLine(x, y, line_offset, text, r, g, b, a, duration);

/**
 * Draw text on the screen, starting on the position of origin.
 * @param {Vector} origin
 * @param {string} text
 * @param {bool} use_view_check
 * @param {float} duration
 */
function DebugDrawText(origin, text, use_view_check, duration);

/**
 * Dumps a scope's contents and expands all tables and arrays.
 * @param {integer} indentation
 * @param {table} scope
 */
function __DumpScope(indentation, scope);

/**
 * Dumps information about a class or instance.
 * @param {any} object
 */
function DumpObject(object);

/**
 * Prints message to console without any line feed after.
 * @param {any} message
 */
function Msg(message);

/**
 * Prints message to console with C style formatting. Line feed not included.
 * @param {string} format
 * @varargs {any}
 */
function printf(format, ...);

/**
 * Prints message to console with a line feed after.
 * @param {any} message
 */
function printl(message);

/**
 * Identical to print.
 * @param {any} message
 */
function realPrint(message);

/**
 * Have the specified player send a message to chat.
 * @param {CTFPlayer} player
 * @param {string} message
 * @param {bool} team_only
 */
function Say(player, message, team_only);

/**
 * Displays a HUD message defined in scripts/titles.txt to all clients.
 * @type {function}
 * @param {string} message
 */
function ShowMessage(message);


// ============================================================
// GLOBAL INSTANCES
// ============================================================

/**
 * Provides an interface to read and change the values of console variables.
 * @type {Convars}
 * @const
 */
Convars <- Convars()

/**
 * Provides access to currently spawned entities.
 * @type {CEntities}
 * @const
 */
Entities <- CEntities()

/**
 * Allows manipulation of entity output data.
 * @type {CScriptEntityOutputs}
 * @const
 */
EntityOutputs <- CScriptEntityOutputs()

/**
 * Provides access to the maps NavMesh and NavAreas.
 * @type {CNavMesh}
 * @const
 */
NavMesh <- CNavMesh()

/**
 * Allows reading and updating the network properties of an entity.
 * @type {CNetPropManager}
 * @const
 */
NetProps <- CNetPropManager()

/**
 * Tracks if any player is using voice and for how long.
 * @type {CPlayerVoiceListener}
 * @const
 */
PlayerVoiceListener <- CPlayerVoiceListener()

/**
 * Contains the printed strings from the script_help command.
 * @type {table}
 */
Documentation <- {}

/**
 * Table of registered game event callbacks.
 * @type {table}
 */
GameEventCallbacks <- {}

/**
 * Table of registered game event callbacks.
 * @type {integer}
 */
print_indent <- 0

/**
 * @type {table}
 */
PublishedHelp <- {}

/**
 * Table of registered script event callbacks.
 * @type {table}
 */
ScriptEventCallbacks <- {}

/**
 * Table of registered script hook callbacks.
 * @type {table}
 */
ScriptHookCallbacks <- {}

/**
 * Enumerations for various function arguments or netprops.
 * @type {table}
 */
Constants <- {
	EBotType = {
		TF_BOT_TYPE = 1337
	}
	ECollisionGroup = {
		COLLISION_GROUP_NONE = 0
		COLLISION_GROUP_DEBRIS = 1
		COLLISION_GROUP_DEBRIS_TRIGGER = 2
		COLLISION_GROUP_INTERACTIVE_DEBRIS = 3
		COLLISION_GROUP_INTERACTIVE = 4
		COLLISION_GROUP_PLAYER = 5
		COLLISION_GROUP_BREAKABLE_GLASS = 6
		COLLISION_GROUP_VEHICLE = 7
		COLLISION_GROUP_PLAYER_MOVEMENT = 8
		COLLISION_GROUP_NPC = 9
		COLLISION_GROUP_IN_VEHICLE = 10
		COLLISION_GROUP_WEAPON = 11
		COLLISION_GROUP_VEHICLE_CLIP = 12
		COLLISION_GROUP_PROJECTILE = 13
		COLLISION_GROUP_DOOR_BLOCKER = 14
		COLLISION_GROUP_PASSABLE_DOOR = 15
		COLLISION_GROUP_DISSOLVING = 16
		COLLISION_GROUP_PUSHAWAY = 17
		COLLISION_GROUP_NPC_ACTOR = 18
		COLLISION_GROUP_NPC_SCRIPTED = 19
		LAST_SHARED_COLLISION_GROUP = 20
	}
	ECritType = {
		CRIT_NONE = 0
		CRIT_MINI = 1
		CRIT_FULL = 2
	}
	EHitGroup = {
		HITGROUP_GENERIC = null
		HITGROUP_HEAD = 1
		HITGROUP_CHEST = 2
		HITGROUP_STOMACH = 3
		HITGROUP_LEFTARM = 4
		HITGROUP_RIGHTARM = 5
		HITGROUP_LEFTLEG = 6
		HITGROUP_RIGHTLEG = 7
		HITGROUP_GEAR = 10
	}
	EHoliday = {
		kHoliday_None = 0
		kHoliday_TFBirthday = 1
		kHoliday_Halloween = 2
		kHoliday_Christmas = 3
		kHoliday_CommunityUpdate = 4
		kHoliday_EOTL = 5
		kHoliday_Valentines = 6
		kHoliday_MeetThePyro = 7
		kHoliday_FullMoon = 8
		kHoliday_HalloweenOrFullMoon = 9
		kHoliday_HalloweenOrFullMoonOrValentines = 10
		kHoliday_AprilFools = 11
		kHoliday_Soldier = 12
		kHoliday_Summer = 13
		kHolidayCount = 14
	}
	EHudNotify = {
		HUD_PRINTNOTIFY = 1
		HUD_PRINTCONSOLE = 2
		HUD_PRINTTALK = 3
		HUD_PRINTCENTER = 4
	}
	EMoveCollide = {
		MOVECOLLIDE_DEFAULT = 0
		MOVECOLLIDE_FLY_BOUNCE = 1
		MOVECOLLIDE_FLY_CUSTOM = 2
		MOVECOLLIDE_FLY_SLIDE = 3
		MOVECOLLIDE_MAX_BITS = 3
		MOVECOLLIDE_COUNT = 4
	}
	EMoveType = {
		MOVETYPE_NONE = 0
		MOVETYPE_ISOMETRIC = 1
		MOVETYPE_WALK = 2
		MOVETYPE_STEP = 3
		MOVETYPE_FLY = 4
		MOVETYPE_FLYGRAVITY = 5
		MOVETYPE_VPHYSICS = 6
		MOVETYPE_PUSH = 7
		MOVETYPE_NOCLIP = 8
		MOVETYPE_LADDER = 9
		MOVETYPE_OBSERVER = 10
		MOVETYPE_CUSTOM = 11
		MOVETYPE_LAST = 11
	}
	ENavCornerType = {
		NORTH_WEST = 0
		NORTH_EAST = 1
		SOUTH_EAST = 2
		SOUTH_WEST = 3
		NUM_CORNERS = 4
	}
	ENavDirType = {
		NORTH = 0
		EAST = 1
		SOUTH = 2
		WEST = 3
		NUM_DIRECTIONS = 4
	}
	ENavRelativeDirType = {
		FORWARD = 0
		RIGHT = 1
		BACKWARD = 2
		LEFT = 3
		UP = 4
		DOWN = 5
		NUM_RELATIVE_DIRECTIONS = 6
	}
	ENavTraverseType = {
		GO_NORTH = 0
		GO_EAST = 1
		GO_SOUTH = 2
		GO_WEST = 3
		GO_LADDER_UP = 4
		GO_LADDER_DOWN = 5
		GO_JUMP = 6
		GO_ELEVATOR_UP = 7
		GO_ELEVATOR_DOWN = 8
		NUM_TRAVERSE_TYPES = 9
	}
	ERenderFx = {
		kRenderFxNone = 0
		kRenderFxPulseSlow = 1
		kRenderFxPulseFast = 2
		kRenderFxPulseSlowWide = 3
		kRenderFxPulseFastWide = 4
		kRenderFxFadeSlow = 5
		kRenderFxFadeFast = 6
		kRenderFxSolidSlow = 7
		kRenderFxSolidFast = 8
		kRenderFxStrobeSlow = 9
		kRenderFxStrobeFast = 10
		kRenderFxStrobeFaster = 11
		kRenderFxFlickerSlow = 12
		kRenderFxFlickerFast = 13
		kRenderFxNoDissipation = 14
		kRenderFxDistort = 15
		kRenderFxHologram = 16
		kRenderFxExplode = 17
		kRenderFxGlowShell = 18
		kRenderFxClampMinScale = 19
		kRenderFxEnvRain = 20
		kRenderFxEnvSnow = 21
		kRenderFxSpotlight = 22
		kRenderFxRagdoll = 23
		kRenderFxPulseFastWider = 24
		kRenderFxMax = 25
	}
	ERenderMode = {
		kRenderNormal = 0
		kRenderTransColor = 1
		kRenderTransTexture = 2
		kRenderGlow = 3
		kRenderTransAlpha = 4
		kRenderTransAdd = 5
		kRenderEnvironmental = 6
		kRenderTransAddFrameBlend = 7
		kRenderTransAlphaAdd = 8
		kRenderWorldGlow = 9
		kRenderNone = 10
		kRenderModeCount = 11
	}
	ERoundState = {
		GR_STATE_INIT = 0
		GR_STATE_PREGAME = 1
		GR_STATE_STARTGAME = 2
		GR_STATE_PREROUND = 3
		GR_STATE_RND_RUNNING = 4
		GR_STATE_TEAM_WIN = 5
		GR_STATE_RESTART = 6
		GR_STATE_STALEMATE = 7
		GR_STATE_GAME_OVER = 8
		// GR_STATE_BONUS = 9
		// GR_STATE_BETWEEN_RNDS = 10
		GR_NUM_ROUND_STATES = 11
	}
	EScriptRecipientFilter = {
		RECIPIENT_FILTER_DEFAULT = 0
		RECIPIENT_FILTER_PAS_ATTENUATION = 1
		RECIPIENT_FILTER_PAS = 2
		RECIPIENT_FILTER_PVS = 3
		RECIPIENT_FILTER_SINGLE_PLAYER = 4
		RECIPIENT_FILTER_GLOBAL = 5
		RECIPIENT_FILTER_TEAM = 6
	}
	ESolidType = {
		SOLID_NONE = 0
		SOLID_BSP = 1
		SOLID_BBOX = 2
		SOLID_OBB = 3
		SOLID_OBB_YAW = 4
		SOLID_CUSTOM = 5
		SOLID_VPHYSICS = 6
		SOLID_LAST = 7
	}
	ESpectatorMode = {
		OBS_MODE_NONE = 0
		OBS_MODE_DEATHCAM = 1
		OBS_MODE_FREEZECAM = 2
		OBS_MODE_FIXED = 3
		OBS_MODE_IN_EYE = 4
		OBS_MODE_CHASE = 5
		OBS_MODE_POI = 6
		OBS_MODE_ROAMING = 7
		NUM_OBSERVER_MODES = 8
	}
	EStopwatchState = {
		STOPWATCH_CAPTURE_TIME_NOT_SET = 0
		STOPWATCH_RUNNING = 1
		STOPWATCH_OVERTIME = 2
	}
	ETFBotDifficultyType = {
		EASY = 0
		NORMAL = 1
		HARD = 2
		EXPERT = 3
		NUM_DIFFICULTY_LEVELS = 4
		UNDEFINED = -1
	}
	ETFClass = {
		TF_CLASS_UNDEFINED = 0
		TF_CLASS_SCOUT = 1
		TF_CLASS_SNIPER = 2
		TF_CLASS_SOLDIER = 3
		TF_CLASS_DEMOMAN = 4
		TF_CLASS_MEDIC = 5
		TF_CLASS_HEAVYWEAPONS = 6
		TF_CLASS_PYRO = 7
		TF_CLASS_SPY = 8
		TF_CLASS_ENGINEER = 9
		TF_CLASS_CIVILIAN = 10
		TF_CLASS_COUNT_ALL = 11
		TF_CLASS_RANDOM = 12
	}
	ETFCond = {
		TF_COND_AIMING = 0
		TF_COND_ZOOMED = 1
		TF_COND_DISGUISING = 2
		TF_COND_DISGUISED = 3
		TF_COND_STEALTHED = 4
		TF_COND_INVULNERABLE = 5
		TF_COND_TELEPORTED = 6
		TF_COND_TAUNTING = 7
		TF_COND_INVULNERABLE_WEARINGOFF = 8
		TF_COND_STEALTHED_BLINK = 9
		TF_COND_SELECTED_TO_TELEPORT = 10
		TF_COND_CRITBOOSTED = 11
		TF_COND_TMPDAMAGEBONUS = 12
		TF_COND_FEIGN_DEATH = 13
		TF_COND_PHASE = 14
		TF_COND_STUNNED = 15
		TF_COND_OFFENSEBUFF = 16
		TF_COND_SHIELD_CHARGE = 17
		TF_COND_DEMO_BUFF = 18
		TF_COND_ENERGY_BUFF = 19
		TF_COND_RADIUSHEAL = 20
		TF_COND_HEALTH_BUFF = 21
		TF_COND_BURNING = 22
		TF_COND_HEALTH_OVERHEALED = 23
		TF_COND_URINE = 24
		TF_COND_BLEEDING = 25
		TF_COND_DEFENSEBUFF = 26
		TF_COND_MAD_MILK = 27
		TF_COND_MEGAHEAL = 28
		TF_COND_REGENONDAMAGEBUFF = 29
		TF_COND_MARKEDFORDEATH = 30
		TF_COND_NOHEALINGDAMAGEBUFF = 31
		TF_COND_SPEED_BOOST = 32
		TF_COND_CRITBOOSTED_PUMPKIN = 33
		TF_COND_CRITBOOSTED_USER_BUFF = 34
		TF_COND_CRITBOOSTED_DEMO_CHARGE = 35
		TF_COND_SODAPOPPER_HYPE = 36
		TF_COND_CRITBOOSTED_FIRST_BLOOD = 37
		TF_COND_CRITBOOSTED_BONUS_TIME = 38
		TF_COND_CRITBOOSTED_CTF_CAPTURE = 39
		TF_COND_CRITBOOSTED_ON_KILL = 40
		TF_COND_CANNOT_SWITCH_FROM_MELEE = 41
		TF_COND_DEFENSEBUFF_NO_CRIT_BLOCK = 42
		TF_COND_REPROGRAMMED = 43
		TF_COND_CRITBOOSTED_RAGE_BUFF = 44
		TF_COND_DEFENSEBUFF_HIGH = 45
		TF_COND_SNIPERCHARGE_RAGE_BUFF = 46
		TF_COND_DISGUISE_WEARINGOFF = 47
		TF_COND_MARKEDFORDEATH_SILENT = 48
		TF_COND_DISGUISED_AS_DISPENSER = 49
		TF_COND_SAPPED = 50
		TF_COND_INVULNERABLE_HIDE_UNLESS_DAMAGED = 51
		TF_COND_INVULNERABLE_USER_BUFF = 52
		TF_COND_HALLOWEEN_BOMB_HEAD = 53
		TF_COND_HALLOWEEN_THRILLER = 54
		TF_COND_RADIUSHEAL_ON_DAMAGE = 55
		TF_COND_CRITBOOSTED_CARD_EFFECT = 56
		TF_COND_INVULNERABLE_CARD_EFFECT = 57
		TF_COND_MEDIGUN_UBER_BULLET_RESIST = 58
		TF_COND_MEDIGUN_UBER_BLAST_RESIST = 59
		TF_COND_MEDIGUN_UBER_FIRE_RESIST = 60
		TF_COND_MEDIGUN_SMALL_BULLET_RESIST = 61
		TF_COND_MEDIGUN_SMALL_BLAST_RESIST = 62
		TF_COND_MEDIGUN_SMALL_FIRE_RESIST = 63
		TF_COND_STEALTHED_USER_BUFF = 64
		TF_COND_MEDIGUN_DEBUFF = 65
		TF_COND_STEALTHED_USER_BUFF_FADING = 66
		TF_COND_BULLET_IMMUNE = 67
		TF_COND_BLAST_IMMUNE = 68
		TF_COND_FIRE_IMMUNE = 69
		TF_COND_PREVENT_DEATH = 70
		TF_COND_MVM_BOT_STUN_RADIOWAVE = 71
		TF_COND_HALLOWEEN_SPEED_BOOST = 72
		TF_COND_HALLOWEEN_QUICK_HEAL = 73
		TF_COND_HALLOWEEN_GIANT = 74
		TF_COND_HALLOWEEN_TINY = 75
		TF_COND_HALLOWEEN_IN_HELL = 76
		TF_COND_HALLOWEEN_GHOST_MODE = 77
		TF_COND_MINICRITBOOSTED_ON_KILL = 78
		TF_COND_OBSCURED_SMOKE = 79
		TF_COND_PARACHUTE_ACTIVE = 80
		TF_COND_BLASTJUMPING = 81
		TF_COND_HALLOWEEN_KART = 82
		TF_COND_HALLOWEEN_KART_DASH = 83
		TF_COND_BALLOON_HEAD = 84
		TF_COND_MELEE_ONLY = 85
		TF_COND_SWIMMING_CURSE = 86
		TF_COND_FREEZE_INPUT = 87
		TF_COND_HALLOWEEN_KART_CAGE = 88
		TF_COND_DONOTUSE_0 = 89
		TF_COND_RUNE_STRENGTH = 90
		TF_COND_RUNE_HASTE = 91
		TF_COND_RUNE_REGEN = 92
		TF_COND_RUNE_RESIST = 93
		TF_COND_RUNE_VAMPIRE = 94
		TF_COND_RUNE_REFLECT = 95
		TF_COND_RUNE_PRECISION = 96
		TF_COND_RUNE_AGILITY = 97
		TF_COND_GRAPPLINGHOOK = 98
		TF_COND_GRAPPLINGHOOK_SAFEFALL = 99
		TF_COND_GRAPPLINGHOOK_LATCHED = 100
		TF_COND_GRAPPLINGHOOK_BLEEDING = 101
		TF_COND_AFTERBURN_IMMUNE = 102
		TF_COND_RUNE_KNOCKOUT = 103
		TF_COND_RUNE_IMBALANCE = 104
		TF_COND_CRITBOOSTED_RUNE_TEMP = 105
		TF_COND_PASSTIME_INTERCEPTION = 106
		TF_COND_SWIMMING_NO_EFFECTS = 107
		TF_COND_PURGATORY = 108
		TF_COND_RUNE_KING = 109
		TF_COND_RUNE_PLAGUE = 110
		TF_COND_RUNE_SUPERNOVA = 111
		TF_COND_PLAGUE = 112
		TF_COND_KING_BUFFED = 113
		TF_COND_TEAM_GLOWS = 114
		TF_COND_KNOCKED_INTO_AIR = 115
		TF_COND_COMPETITIVE_WINNER = 116
		TF_COND_COMPETITIVE_LOSER = 117
		TF_COND_HEALING_DEBUFF = 118
		TF_COND_PASSTIME_PENALTY_DEBUFF = 119
		TF_COND_GRAPPLED_TO_PLAYER = 120
		TF_COND_GRAPPLED_BY_PLAYER = 121
		TF_COND_PARACHUTE_DEPLOYED = 122
		TF_COND_GAS = 123
		TF_COND_BURNING_PYRO = 124
		TF_COND_ROCKETPACK = 125
		TF_COND_LOST_FOOTING = 126
		TF_COND_AIR_CURRENT = 127
		TF_COND_HALLOWEEN_HELL_HEAL = 128
		TF_COND_POWERUPMODE_DOMINANT = 129
		TF_COND_IMMUNE_TO_PUSHBACK = 130
		TF_COND_INVALID = -1
	}
	ETFDmgCustom = {
		TF_DMG_CUSTOM_NONE = 0
		TF_DMG_CUSTOM_HEADSHOT = 1
		TF_DMG_CUSTOM_BACKSTAB = 2
		TF_DMG_CUSTOM_BURNING = 3
		TF_DMG_WRENCH_FIX = 4
		TF_DMG_CUSTOM_MINIGUN = 5
		TF_DMG_CUSTOM_SUICIDE = 6
		TF_DMG_CUSTOM_TAUNTATK_HADOUKEN = 7
		TF_DMG_CUSTOM_BURNING_FLARE = 8
		TF_DMG_CUSTOM_TAUNTATK_HIGH_NOON = 9
		TF_DMG_CUSTOM_TAUNTATK_GRAND_SLAM = 10
		TF_DMG_CUSTOM_PENETRATE_MY_TEAM = 11
		TF_DMG_CUSTOM_PENETRATE_ALL_PLAYERS = 12
		TF_DMG_CUSTOM_TAUNTATK_FENCING = 13
		TF_DMG_CUSTOM_PENETRATE_NONBURNING_TEAMMATE = 14
		TF_DMG_CUSTOM_TAUNTATK_ARROW_STAB = 15
		TF_DMG_CUSTOM_TELEFRAG = 16
		TF_DMG_CUSTOM_BURNING_ARROW = 17
		TF_DMG_CUSTOM_FLYINGBURN = 18
		TF_DMG_CUSTOM_PUMPKIN_BOMB = 19
		TF_DMG_CUSTOM_DECAPITATION = 20
		TF_DMG_CUSTOM_TAUNTATK_GRENADE = 21
		TF_DMG_CUSTOM_BASEBALL = 22
		TF_DMG_CUSTOM_CHARGE_IMPACT = 23
		TF_DMG_CUSTOM_TAUNTATK_BARBARIAN_SWING = 24
		TF_DMG_CUSTOM_AIR_STICKY_BURST = 25
		TF_DMG_CUSTOM_DEFENSIVE_STICKY = 26
		TF_DMG_CUSTOM_PICKAXE = 27
		TF_DMG_CUSTOM_ROCKET_DIRECTHIT = 28
		TF_DMG_CUSTOM_TAUNTATK_UBERSLICE = 29
		TF_DMG_CUSTOM_PLAYER_SENTRY = 30
		TF_DMG_CUSTOM_STANDARD_STICKY = 31
		TF_DMG_CUSTOM_SHOTGUN_REVENGE_CRIT = 32
		TF_DMG_CUSTOM_TAUNTATK_ENGINEER_GUITAR_SMASH = 33
		TF_DMG_CUSTOM_BLEEDING = 34
		TF_DMG_CUSTOM_GOLD_WRENCH = 35
		TF_DMG_CUSTOM_CARRIED_BUILDING = 36
		TF_DMG_CUSTOM_COMBO_PUNCH = 37
		TF_DMG_CUSTOM_TAUNTATK_ENGINEER_ARM_KILL = 38
		TF_DMG_CUSTOM_FISH_KILL = 39
		TF_DMG_CUSTOM_TRIGGER_HURT = 40
		TF_DMG_CUSTOM_DECAPITATION_BOSS = 41
		TF_DMG_CUSTOM_STICKBOMB_EXPLOSION = 42
		TF_DMG_CUSTOM_AEGIS_ROUND = 43
		TF_DMG_CUSTOM_FLARE_EXPLOSION = 44
		TF_DMG_CUSTOM_BOOTS_STOMP = 45
		TF_DMG_CUSTOM_PLASMA = 46
		TF_DMG_CUSTOM_PLASMA_CHARGED = 47
		TF_DMG_CUSTOM_PLASMA_GIB = 48
		TF_DMG_CUSTOM_PRACTICE_STICKY = 49
		TF_DMG_CUSTOM_EYEBALL_ROCKET = 50
		TF_DMG_CUSTOM_HEADSHOT_DECAPITATION = 51
		TF_DMG_CUSTOM_TAUNTATK_ARMAGEDDON = 52
		TF_DMG_CUSTOM_FLARE_PELLET = 53
		TF_DMG_CUSTOM_CLEAVER = 54
		TF_DMG_CUSTOM_CLEAVER_CRIT = 55
		TF_DMG_CUSTOM_SAPPER_RECORDER_DEATH = 56
		TF_DMG_CUSTOM_MERASMUS_PLAYER_BOMB = 57
		TF_DMG_CUSTOM_MERASMUS_GRENADE = 58
		TF_DMG_CUSTOM_MERASMUS_ZAP = 59
		TF_DMG_CUSTOM_MERASMUS_DECAPITATION = 60
		TF_DMG_CUSTOM_CANNONBALL_PUSH = 61
		TF_DMG_CUSTOM_TAUNTATK_ALLCLASS_GUITAR_RIFF = 62
		TF_DMG_CUSTOM_THROWABLE = 63
		TF_DMG_CUSTOM_THROWABLE_KILL = 64
		TF_DMG_CUSTOM_SPELL_TELEPORT = 65
		TF_DMG_CUSTOM_SPELL_SKELETON = 66
		TF_DMG_CUSTOM_SPELL_MIRV = 67
		TF_DMG_CUSTOM_SPELL_METEOR = 68
		TF_DMG_CUSTOM_SPELL_LIGHTNING = 69
		TF_DMG_CUSTOM_SPELL_FIREBALL = 70
		TF_DMG_CUSTOM_SPELL_MONOCULUS = 71
		TF_DMG_CUSTOM_SPELL_BLASTJUMP = 72
		TF_DMG_CUSTOM_SPELL_BATS = 73
		TF_DMG_CUSTOM_SPELL_TINY = 74
		TF_DMG_CUSTOM_KART = 75
		TF_DMG_CUSTOM_GIANT_HAMMER = 76
		TF_DMG_CUSTOM_RUNE_REFLECT = 77
		TF_DMG_CUSTOM_DRAGONS_FURY_IGNITE = 78
		TF_DMG_CUSTOM_DRAGONS_FURY_BONUS_BURNING = 79
		TF_DMG_CUSTOM_SLAP_KILL = 80
		TF_DMG_CUSTOM_CROC = 81
		TF_DMG_CUSTOM_TAUNTATK_GASBLAST = 82
		TF_DMG_CUSTOM_AXTINGUISHER_BOOSTED = 83
		TF_DMG_CUSTOM_KRAMPUS_MELEE = 84
		TF_DMG_CUSTOM_KRAMPUS_RANGED = 85
		TF_DMG_CUSTOM_TAUNTATK_TRICKSHOT = 86
		TF_DMG_CUSTOM_NUTCRACKER = 87
		TF_DMG_CUSTOM_END = 88
	}
	ETFTeam = {
		TEAM_UNASSIGNED = null
		TEAM_SPECTATOR = 1
		TF_TEAM_PVE_DEFENDERS = 2
		TF_TEAM_RED = 2
		TF_TEAM_BLUE = 3
		TF_TEAM_PVE_INVADERS = 3
		TF_TEAM_COUNT = 4
		TF_TEAM_PVE_INVADERS_GIANTS = 4
		// TF_TEAM_HALLOWEEN = 5
		TEAM_ANY = -2
		TEAM_INVALID = -1
	}
	Math = {
		Zero = 0
		Epsilon = 1.19209e-07
		GoldenRatio = 1.61803
		One = 1
		Sqrt2 = 1.41421
		Sqrt3 = 1.73205
		E = 2.71828
		Pi = 3.14159
		Tau = 6.28319
	}
	Server = {
		ConstantNamingConvention = "Constants are named as follows: F -> flags, E -> enums, (nothing) -> random values/constants",
		DIST_EPSILON = 0.03125
		MAX_PLAYERS = 101
		MAX_EDICTS = 2048
	}
	FButtons = {
		IN_ATTACK = 1
		IN_JUMP = 2
		IN_DUCK = 4
		IN_FORWARD = 8
		IN_BACK = 16
		IN_USE = 32
		IN_CANCEL = 64
		IN_LEFT = 128
		IN_RIGHT = 256
		IN_MOVELEFT = 512
		IN_MOVERIGHT = 1024
		IN_ATTACK2 = 2048
		IN_RUN = 4096
		IN_RELOAD = 8192
		IN_ALT1 = 16384
		IN_ALT2 = 32768
		IN_SCORE = 65536
		IN_SPEED = 131072
		IN_WALK = 262144
		IN_ZOOM = 524288
		IN_WEAPON1 = 1048576
		IN_WEAPON2 = 2097152
		IN_BULLRUSH = 4194304
		IN_GRENADE1 = 8388608
		IN_GRENADE2 = 16777216
		IN_ATTACK3 = 33554432
	}
	FContents = {
		CONTENTS_EMPTY = null
		CONTENTS_SOLID = 1
		CONTENTS_WINDOW = 2
		CONTENTS_AUX = 4
		CONTENTS_GRATE = 8
		CONTENTS_SLIME = 16
		CONTENTS_WATER = 32
		CONTENTS_BLOCKLOS = 64
		CONTENTS_OPAQUE = 128
		LAST_VISIBLE_CONTENTS = 128
		ALL_VISIBLE_CONTENTS = 255
		CONTENTS_TESTFOGVOLUME = 256
		CONTENTS_UNUSED = 512
		CONTENTS_UNUSED6 = 1024
		CONTENTS_TEAM1 = 2048
		CONTENTS_TEAM2 = 4096
		CONTENTS_IGNORE_NODRAW_OPAQUE = 8192
		CONTENTS_MOVEABLE = 16384
		CONTENTS_AREAPORTAL = 32768
		CONTENTS_PLAYERCLIP = 65536
		CONTENTS_MONSTERCLIP = 131072
		CONTENTS_CURRENT_0 = 262144
		CONTENTS_CURRENT_90 = 524288
		CONTENTS_CURRENT_180 = 1048576
		CONTENTS_CURRENT_270 = 2097152
		CONTENTS_CURRENT_UP = 4194304
		CONTENTS_CURRENT_DOWN = 8388608
		CONTENTS_ORIGIN = 16777216
		CONTENTS_MONSTER = 33554432
		CONTENTS_DEBRIS = 67108864
		CONTENTS_DETAIL = 134217728
		CONTENTS_TRANSLUCENT = 268435456
		CONTENTS_LADDER = 536870912
		CONTENTS_HITBOX = 1073741824
	}
	FDmgType = {
		DMG_GENERIC = null
		DMG_CRUSH = 1
		DMG_BULLET = 2
		DMG_SLASH = 4
		DMG_BURN = 8
		DMG_VEHICLE = 16
		DMG_FALL = 32
		DMG_BLAST = 64
		DMG_CLUB = 128
		DMG_SHOCK = 256
		DMG_SONIC = 512
		DMG_ENERGYBEAM = 1024
		DMG_PREVENT_PHYSICS_FORCE = 2048
		DMG_NEVERGIB = 4096
		DMG_ALWAYSGIB = 8192
		DMG_DROWN = 16384
		DMG_PARALYZE = 32768
		DMG_NERVEGAS = 65536
		DMG_POISON = 131072
		DMG_RADIATION = 262144
		DMG_DROWNRECOVER = 524288
		DMG_ACID = 1048576
		DMG_SLOWBURN = 2097152
		DMG_REMOVENORAGDOLL = 4194304
		DMG_PHYSGUN = 8388608
		DMG_PLASMA = 16777216
		DMG_AIRBOAT = 33554432
		DMG_DISSOLVE = 67108864
		DMG_BLAST_SURFACE = 134217728
		DMG_DIRECT = 268435456
		DMG_BUCKSHOT = 536870912
	}
	FEntityEffects = {
		EF_BONEMERGE = 1
		EF_BRIGHTLIGHT = 2
		EF_DIMLIGHT = 4
		EF_NOINTERP = 8
		EF_MAX_BITS = 10
		EF_NOSHADOW = 16
		EF_NODRAW = 32
		EF_NORECEIVESHADOW = 64
		EF_BONEMERGE_FASTCULL = 128
		EF_ITEM_BLINK = 256
		EF_PARENT_ANIMATES = 512
	}
	FEntityEFlags = {
		EFL_KILLME = 1
		EFL_DORMANT = 2
		EFL_NOCLIP_ACTIVE = 4
		EFL_SETTING_UP_BONES = 8
		EFL_HAS_PLAYER_CHILD = 16
		EFL_KEEP_ON_RECREATE_ENTITIES = 16
		EFL_DIRTY_SHADOWUPDATE = 32
		EFL_NOTIFY = 64
		EFL_FORCE_CHECK_TRANSMIT = 128
		EFL_BOT_FROZEN = 256
		EFL_SERVER_ONLY = 512
		EFL_NO_AUTO_EDICT_ATTACH = 1024
		EFL_DIRTY_ABSTRANSFORM = 2048
		EFL_DIRTY_ABSVELOCITY = 4096
		EFL_DIRTY_ABSANGVELOCITY = 8192
		EFL_DIRTY_SURROUNDING_COLLISION_BOUNDS = 16384
		EFL_DIRTY_SPATIAL_PARTITION = 32768
		EFL_FORCE_ALLOW_MOVEPARENT = 65536
		EFL_IN_SKYBOX = 131072
		EFL_USE_PARTITION_WHEN_NOT_SOLID = 262144
		EFL_TOUCHING_FLUID = 524288
		EFL_IS_BEING_LIFTED_BY_BARNACLE = 1048576
		EFL_NO_ROTORWASH_PUSH = 2097152
		EFL_NO_THINK_FUNCTION = 4194304
		EFL_NO_GAME_PHYSICS_SIMULATION = 8388608
		EFL_CHECK_UNTOUCH = 16777216
		EFL_DONTBLOCKLOS = 33554432
		EFL_DONTWALKON = 67108864
		EFL_NO_DISSOLVE = 134217728
		EFL_NO_MEGAPHYSCANNON_RAGDOLL = 268435456
		EFL_NO_WATER_VELOCITY_CHANGE = 536870912
		EFL_NO_PHYSCANNON_INTERACTION = 1073741824
		EFL_NO_DAMAGE_FORCES = 2147483648
	}
	FHideHUD = {
		HIDEHUD_WEAPONSELECTION = 1
		HIDEHUD_FLASHLIGHT = 2
		HIDEHUD_ALL = 4
		HIDEHUD_HEALTH = 8
		HIDEHUD_PLAYERDEAD = 16
		HIDEHUD_BITCOUNT = 18
		HIDEHUD_NEEDSUIT = 32
		HIDEHUD_MISCSTATUS = 64
		HIDEHUD_CHAT = 128
		HIDEHUD_CROSSHAIR = 256
		HIDEHUD_VEHICLE_CROSSHAIR = 512
		HIDEHUD_INVEHICLE = 1024
		HIDEHUD_BONUS_PROGRESS = 2048
		HIDEHUD_BUILDING_STATUS = 4096
		HIDEHUD_CLOAK_AND_FEIGN = 8192
		HIDEHUD_PIPES_AND_CHARGE = 16384
		HIDEHUD_METAL = 32768
		HIDEHUD_TARGET_ID = 65536
		HIDEHUD_MATCH_STATUS = 131072
	}
	FNavAttributeType = {
		NAV_MESH_INVALID = 0
		NAV_MESH_CROUCH = 1
		NAV_MESH_JUMP = 2
		NAV_MESH_PRECISE = 4
		NAV_MESH_NO_JUMP = 8
		NAV_MESH_STOP = 16
		NAV_MESH_RUN = 32
		NAV_MESH_WALK = 64
		NAV_MESH_AVOID = 128
		NAV_MESH_TRANSIENT = 256
		NAV_MESH_DONT_HIDE = 512
		NAV_MESH_STAND = 1024
		NAV_MESH_NO_HOSTAGES = 2048
		NAV_MESH_STAIRS = 4096
		NAV_MESH_NO_MERGE = 8192
		NAV_MESH_OBSTACLE_TOP = 16384
		NAV_MESH_CLIFF = 32768
		NAV_MESH_FIRST_CUSTOM = 65536
		NAV_MESH_LAST_CUSTOM = 67108864
		NAV_MESH_FUNC_COST = 536870912
		NAV_MESH_HAS_ELEVATOR = 1073741824
		NAV_MESH_NAV_BLOCKER = 2147483648
	}
	FPlayer = {
		FL_ONGROUND = 1
		FL_DUCKING = 2
		FL_ANIMDUCKING = 4
		FL_WATERJUMP = 8
		PLAYER_FLAG_BITS = 11
		FL_ONTRAIN = 16
		FL_INRAIN = 32
		FL_FROZEN = 64
		FL_ATCONTROLS = 128
		FL_CLIENT = 256
		FL_FAKECLIENT = 512
		FL_INWATER = 1024
		FL_FLY = 2048
		FL_SWIM = 4096
		FL_CONVEYOR = 8192
		FL_NPC = 16384
		FL_GODMODE = 32768
		FL_NOTARGET = 65536
		FL_AIMTARGET = 131072
		FL_PARTIALGROUND = 262144
		FL_STATICPROP = 524288
		FL_GRAPHED = 1048576
		FL_GRENADE = 2097152
		FL_STEPMOVEMENT = 4194304
		FL_DONTTOUCH = 8388608
		FL_BASEVELOCITY = 16777216
		FL_WORLDBRUSH = 33554432
		FL_OBJECT = 67108864
		FL_KILLME = 134217728
		FL_ONFIRE = 268435456
		FL_DISSOLVING = 536870912
		FL_TRANSRAGDOLL = 1073741824
		FL_UNBLOCKABLE_BY_PLAYER = 2147483648
	}
	FSolid = {
		FSOLID_CUSTOMRAYTEST = 1
		FSOLID_CUSTOMBOXTEST = 2
		FSOLID_NOT_SOLID = 4
		FSOLID_TRIGGER = 8
		FSOLID_MAX_BITS = 10
		FSOLID_NOT_STANDABLE = 16
		FSOLID_VOLUME_CONTENTS = 32
		FSOLID_FORCE_WORLD_ALIGNED = 64
		FSOLID_USE_TRIGGER_BOUNDS = 128
		FSOLID_ROOT_PARENT_ALIGNED = 256
		FSOLID_TRIGGER_TOUCH_DEBRIS = 512
	}
	FSurf = {
		SURF_LIGHT = 1
		SURF_SKY2D = 2
		SURF_SKY = 4
		SURF_WARP = 8
		SURF_TRANS = 16
		SURF_NOPORTAL = 32
		SURF_TRIGGER = 64
		SURF_NODRAW = 128
		SURF_HINT = 256
		SURF_SKIP = 512
		SURF_NOLIGHT = 1024
		SURF_BUMPLIGHT = 2048
		SURF_NOSHADOWS = 4096
		SURF_NODECALS = 8192
		SURF_NOCHOP = 16384
		SURF_HITBOX = 32768
	}
	FTaunts = {
		TAUNT_BASE_WEAPON = 0
		TAUNT_MISC_ITEM = 1
		TAUNT_SHOW_ITEM = 2
		TAUNT_LONG = 3
		TAUNT_SPECIAL = 4
	}
	FTFBotAttributeType = {
		REMOVE_ON_DEATH = 1
		AGGRESSIVE = 2
		IS_NPC = 4
		SUPPRESS_FIRE = 8
		DISABLE_DODGE = 16
		BECOME_SPECTATOR_ON_DEATH = 32
		QUOTA_MANANGED = 64
		RETAIN_BUILDINGS = 128
		SPAWN_WITH_FULL_CHARGE = 256
		ALWAYS_CRIT = 512
		IGNORE_ENEMIES = 1024
		HOLD_FIRE_UNTIL_FULL_RELOAD = 2048
		PRIORITIZE_DEFENSE = 4096
		ALWAYS_FIRE_WEAPON = 8192
		TELEPORT_TO_HINT = 16384
		MINIBOSS = 32768
		USE_BOSS_HEALTH_BAR = 65536
		IGNORE_FLAG = 131072
		AUTO_JUMP = 262144
		AIR_CHARGE_ONLY = 524288
		PREFER_VACCINATOR_BULLETS = 1048576
		PREFER_VACCINATOR_BLAST = 2097152
		PREFER_VACCINATOR_FIRE = 4194304
		BULLET_IMMUNE = 8388608
		BLAST_IMMUNE = 16777216
		FIRE_IMMUNE = 33554432
		PARACHUTE = 67108864
		PROJECTILE_SHIELD = 134217728
	}
	FTFNavAttributeType = {
		TF_NAV_INVALID = 0
		TF_NAV_BLOCKED = 1
		TF_NAV_SPAWN_ROOM_RED = 2
		TF_NAV_SPAWN_ROOM_BLUE = 4
		TF_NAV_SPAWN_ROOM_EXIT = 8
		TF_NAV_HAS_AMMO = 16
		TF_NAV_HAS_HEALTH = 32
		TF_NAV_CONTROL_POINT = 64
		TF_NAV_BLUE_SENTRY_DANGER = 128
		TF_NAV_RED_SENTRY_DANGER = 256
		TF_NAV_BLUE_SETUP_GATE = 2048
		TF_NAV_RED_SETUP_GATE = 4096
		TF_NAV_BLOCKED_AFTER_POINT_CAPTURE = 8192
		TF_NAV_BLOCKED_UNTIL_POINT_CAPTURE = 16384
		TF_NAV_BLUE_ONE_WAY_DOOR = 32768
		TF_NAV_RED_ONE_WAY_DOOR = 65536
		TF_NAV_WITH_SECOND_POINT = 131072
		TF_NAV_WITH_THIRD_POINT = 262144
		TF_NAV_WITH_FOURTH_POINT = 524288
		TF_NAV_WITH_FIFTH_POINT = 1048576
		TF_NAV_SNIPER_SPOT = 2097152
		TF_NAV_SENTRY_SPOT = 4194304
		TF_NAV_ESCAPE_ROUTE = 8388608
		TF_NAV_ESCAPE_ROUTE_VISIBLE = 16777216
		TF_NAV_NO_SPAWNING = 33554432
		TF_NAV_RESCUE_CLOSET = 67108864
		TF_NAV_BOMB_CAN_DROP_HERE = 134217728
		TF_NAV_DOOR_NEVER_BLOCKS = 268435456
		TF_NAV_DOOR_ALWAYS_BLOCKS = 536870912
		TF_NAV_UNBLOCKABLE = 1073741824
		TF_NAV_PERSISTENT_ATTRIBUTES = 1988098048
	}
}

/** @type {array} */
ScriptDebugDefaultWatchColor <- [0, 192, 0]
/** @type {bool} */
ScriptDebugDrawTextEnabled <- true
/** @type {bool} */
ScriptDebugDrawWatchesEnabled <- true
/** @type {bool} */
ScriptDebugInDebugDraw <- false
/** @type {array} */
ScriptDebugText <- []
/** @type {integer} */
ScriptDebugTextIndent <- 0
/** @type {table} */
ScriptDebugTextFilters <- {}
/** @type {table} */
ScriptDebugTraces <- {}
/** @type {bool} */
ScriptDebugTraceAllOn <- false
/** @type {array} */
ScriptDebugWatches <- []