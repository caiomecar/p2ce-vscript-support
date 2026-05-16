/* 
 * P2CE VScript definitions
 * Generated using https://raw.githubusercontent.com/StrataSource/Wiki/refs/heads/main/dumps/vscript.json as reference
 */

/*
 * =======================
 * GLOBAL FUNCTIONS
 * =======================
 */

/**
 * Activates the specified paint power on all players.
 *
 * @type {function}
 * @param {int} paintType
 */
function ActivatePaint(paintType);

/**
 * Adds a level to the specified branch's list.
 *
 * @type {function}
 * @param {int} branch
 * @param {string} levelName
 */
function AddBranchLevelName(branch, levelName);

/**
 * Adds a name to the coop credits list.
 *
 * @type {function}
 * @param {string} name
 */
function AddCoopCreditsName(name);

/**
 * Create entity by classname, setting the specified key values before spawn.
 *
 * @type {function}
 * @param {classname} className
 * @param {table} entKeyVals
 * @returns {CBaseEntity|null}
 */
function CreateEntityByName(className, entKeyVals);

/**
 * Create a physics prop, setting the specified model name and activity index. Prefer CreateEntityByName() for more flexibility.
 *
 * @type {function}
 * @param {string} classname
 * @param {Vector} origin
 * @param {string} modelName
 * @param {int} activityIndex
 * @returns {CBaseEntity|null}
 */
function CreateProp(classname, origin, modelName, activityIndex);

/**
 * Create a scene entity to play the specified scene.
 *
 * @type {function}
 * @param {string} filename
 * @returns {CBaseEntity|null}
 */
function CreateSceneEntity(filename);

/**
 * Deactivates all the paints on all players.
 *
 * @type {function}
 */
function DeactivateAllPaints();

/**
 * Deactivates the specified paint power on all players.
 *
 * @type {function}
 * @param {int} paintType
 */
function DeactivatePaint(paintType);

/**
 * Draw a debug overlay box.
 *
 * @type {function}
 * @param {Vector} origin
 * @param {Vector} mins
 * @param {Vector} maxes
 * @param {int} r
 * @param {int} g
 * @param {int} b
 * @param {int} a
 * @param {float} duration
 */
function DebugDrawBox(origin, mins, maxes, r, g, b, a, duration);

/**
 * Draw a debug overlay box with angles/
 *
 * @type {function}
 * @param {Vector} origin
 * @param {Vector} mins
 * @param {Vector} maxes
 * @param {Vector} angles
 * @param {int} r
 * @param {int} g
 * @param {int} b
 * @param {int} a
 * @param {float} duration
 */
function DebugDrawBoxAngles(origin, mins, maxes, angles, r, g, b, a, duration);

/**
 * Draw debug overlay entity text.
 *
 * @type {function}
 * @param {int} entityID
 * @param {int} textOffset
 * @param {string} text
 * @param {float} duration
 * @param {int} r
 * @param {int} g
 * @param {int} b
 * @param {int} a
 */
function DebugDrawEntityText(entityID, textOffset, text, duration, r, g, b, a);

/**
 * Draw a debug overlay entity text at position.
 *
 * @type {function}
 * @param {Vector} origin
 * @param {int} textOffset
 * @param {string} text
 * @param {float} duration
 * @param {int} r
 * @param {int} g
 * @param {int} b
 * @param {int} a
 */
function DebugDrawEntityTextAtPosition(origin, textOffset, text, duration, r, g, b, a);

/**
 * Draw debug overlay grid.
 *
 * @type {function}
 * @param {Vector} origin
 */
function DebugDrawGrid(origin);

/**
 * Draw a debug overlay line.
 *
 * @type {function}
 * @param {Vector} p1
 * @param {Vector} p2
 * @param {int} r
 * @param {int} g
 * @param {int} b
 * @param {bool} noDepthTest
 * @param {float} duration
 */
function DebugDrawLine(p1, p2, r, g, b, noDepthTest, duration);

/**
 * Draw debug overlay screen text.
 *
 * @type {function}
 * @param {float} x
 * @param {float} y
 * @param {string} text
 * @param {int} r
 * @param {int} g
 * @param {int} b
 * @param {int} a
 * @param {float} duration
 */
function DebugDrawScreenText(x, y, text, r, g, b, a, duration);

/**
 * Draw debug overlay text.
 *
 * @type {function}
 * @param {Vector} origin
 * @param {string} text
 * @param {bool} viewCheck
 * @param {float} duration
 */
function DebugDrawText(origin, text, viewCheck, duration);

/**
 * Draw a debug overlay triangle.
 *
 * @type {function}
 * @param {Vector} p1
 * @param {Vector} p2
 * @param {Vector} p3
 * @param {int} r
 * @param {int} g
 * @param {int} b
 * @param {int} a
 * @param {bool} noDepthTest
 * @param {float} duration
 */
function DebugDrawTri(p1, p2, p3, r, g, b, a, noDepthTest, duration);

/**
 * Dispatches a one-off particle system, automatically cleaned up once finished.
 *
 * @type {function}
 * @param {string} particleName
 * @param {Vector} origin
 * @param {Vector} angles
 */
function DispatchParticleEffect(particleName, origin, angles);

/**
 * Implementation of IncludeScript(), use that instead.
 *
 * @type {function}
 * @param {string} filename
 * @param {table} scope
 * @returns {bool}
 */
function DoIncludeScript(filename, scope);

/**
 * Generate an entity i/o event, searching by entity name.
 *
 * @type {function}
 * @param {object} target
 * @param {object} action
 * @param {object} value
 * @param {object} delay
 * @param {object} activator
 * @returns {function}
 */
function EntFire(target, action, value, delay, activator);

/**
 * Generate an entity i/o event, directly targeting an entity by reference.
 *
 * @type {function}
 * @param {CBaseEntity|null} target
 * @param {string} input
 * @param {string} parameter
 * @param {float} delay
 * @param {CBaseEntity|null} activator
 * @param {CBaseEntity|null} caller
 */
function EntFireByHandle(target, input, parameter, delay, activator, caller);

/**
 * Finds a portal by linkage ID and portal number. Portal number 1 is the primary portal, 2 is the secondary. Linkage ID should be <255
 *
 * @type {function}
 * @param {int} linkageID
 * @param {int} portalNum
 * @returns {CBaseEntity|null}
 */
function FindPortalByID(linkageID, portalNum);

/**
 * Get the time spent on the server in the last frame
 *
 * @type {function}
 * @returns {float}
 */
function FrameTime();

/**
 * Return the player index of the blue player.
 *
 * @type {function}
 * @returns {int}
 */
function GetBluePlayerIndex();

/**
 * Returns the current chosen level in the hub.
 *
 * @type {function}
 * @param {int} branch
 * @returns {int}
 */
function GetCoopBranchLevelIndex(branch);

/**
 * Section that the coop players have selected to load in the hub.
 *
 * @type {function}
 * @returns {int}
 */
function GetCoopSectionIndex();

/**
 * Gets the level of the 'developer' console variable.
 *
 * @type {function}
 * @returns {int}
 */
function GetDeveloperLevel();

/**
 * Returns which branches should be available in the hub.
 *
 * @type {function}
 * @returns {int}
 */
function GetHighestActiveBranch();

/**
 * Determines which index (by order played) this map is. Returns -1 if entry is not found. -2 if this is not a known community map.
 *
 * @type {function}
 * @returns {int}
 */
function GetMapIndexInPlayOrder();

/**
 * Get the name of the map.
 *
 * @type {function}
 * @returns {string}
 */
function GetMapName();

/**
 * Returns how many maps the player has played through.
 *
 * @type {function}
 * @returns {int}
 */
function GetNumMapsPlayed();

/**
 * Return the player index of the orange player.
 *
 * @type {function}
 * @returns {int}
 */
function GetOrangePlayerIndex();

/**
 * Returns the player (SP Only).
 *
 * @type {function}
 * @returns {CBaseEntity|null}
 */
function GetPlayer();

/**
 * Gets the player by their index. This is a one-based index and must be in the range (1 <= index <= GetPlayerCount())
 *
 * @type {function}
 * @param {int} playerIndex
 * @returns {CBaseEntity|null}
 */
function GetPlayerByIndex(playerIndex);

/**
 * Returns the number of connected clients, this will always be 1 for listen servers
 *
 * @type {function}
 * @returns {int}
 */
function GetPlayerCount();

/**
 * Time that the specified player has been silent on the mic.
 *
 * @type {function}
 * @param {int} player
 * @returns {float}
 */
function GetPlayerSilenceDuration(player);

/**
 * Gives all portal players the paint gun with no active paints.
 *
 * @type {function}
 */
function GivePlayerPaintgun();

/**
 * Give player a monoportal portal gun.
 *
 * @type {function}
 */
function GivePlayerPortalgun();

/**
 * Is this a co-op game?
 *
 * @type {function}
 * @returns {bool}
 */
function IsCoOp();

/**
 * Returns true if the level in the specified branch is completed by either player.
 *
 * @type {function}
 * @param {int} branch
 * @param {int} level
 * @returns {bool}
 */
function IsLevelComplete(branch, level);

/**
 * Is this a multiplayer game?
 *
 * @type {function}
 * @returns {bool}
 */
function IsMultiplayer();

/**
 * Returns true if the level in the specified branch is completed by a specific player.
 *
 * @type {function}
 * @param {int} player
 * @param {int} branch
 * @param {int} level
 * @returns {bool}
 */
function IsPlayerLevelComplete(player, branch, level);

/**
 * Run the single player maps in a continuous loop.
 *
 * @type {function}
 * @returns {bool}
 */
function LoopSinglePlayerMaps();

/**
 * Marks a map as complete for both players.
 *
 * @type {function}
 * @param {string} mapName
 */
function MarkMapComplete(mapName);

/**
 * Precaches a named movie. Only valid to call within the entity's 'Precache' function called on mapspawn.
 *
 * @type {function}
 * @param {string} movieName
 */
function PrecacheMovie(movieName);

/**
 * Generate a random floating point number within a range, inclusive
 *
 * @type {function}
 * @param {float} min
 * @param {float} max
 * @returns {float}
 */
function RandomFloat(min, max);

/**
 * Generate a random integer within a range, inclusive
 *
 * @type {function}
 * @param {int} min
 * @param {int} max
 * @returns {int}
 */
function RandomInt(min, max);

/**
 * Records achievement event or progress.
 *
 * @type {function}
 * @param {string} achievement
 * @param {int} playerIndex
 */
function RecordAchievementEvent(achievement, playerIndex);

/**
 * Pops up the map rating dialog for user input
 *
 * @type {function}
 */
function RequestMapRating();

/**
 * Is the local player using a controller?
 *
 * @type {function}
 * @returns {bool}
 */
function ScriptIsLocalPlayerUsingController();

/**
 * Prints an alert message in the center print method to all players.
 *
 * @type {function}
 * @param {string} message
 */
function ScriptPrintMessageCenterAll(message);

/**
 * Prints an alert message in the center print method to all players, substituting parameters. Can pass null for parameters if you need less than 3.
 *
 * @type {function}
 * @param {string} message
 * @param {string} param1
 * @param {string} param2
 * @param {string} param3
 */
function ScriptPrintMessageCenterAllWithParams(message, param1, param2, param3);

/**
 * Prints an alert message in the center print method to the specified team.
 *
 * @type {function}
 * @param {int} team
 * @param {string} message
 */
function ScriptPrintMessageCenterTeam(team, message);

/**
 * Prints a message in chat to all players.
 *
 * @type {function}
 * @param {string} message
 */
function ScriptPrintMessageChatAll(message);

/**
 * Prints a message in chat to the specified team.
 *
 * @type {function}
 * @param {int} team
 * @param {string} message
 */
function ScriptPrintMessageChatTeam(team, message);

/**
 * Show center print text message.
 *
 * @type {function}
 * @param {string} message
 * @param {float} holdTime
 */
function ScriptShowHudMessageAll(message, holdTime);

/**
 * Bring up the steam overlay and shows the specified URL.  (Full address with protocol type is required, e.g. http://www.steamgames.com/)
 *
 * @type {function}
 * @param {string} url
 * @returns {bool}
 */
function ScriptSteamShowURL(url);

/**
 * Execute the specified console command, as if run by the local player or server host.
 *
 * @type {function}
 * @param {string} command
 */
function SendToConsole(command);

/**
 * Send a string that gets executed on the server as a ServerCommand.
 *
 * @type {function}
 * @param {string} command
 */
function SendToConsoleServer(command);

/**
 * Send an event to Panorama.
 *
 * @type {function}
 * @param {string} eventName
 * @param {string} payload
 */
function SendToPanorama(eventName, payload);

/**
 * Set the level of an audio ducking channel
 *
 * @type {function}
 * @param {string} layer
 * @param {string} mixGroup
 * @param {float} factor
 */
function SetDucking(layer, mixGroup, factor);

/**
 * Adds the current map to the play order and returns the new index therein. Returns -2 if this is not a known community map.
 *
 * @type {function}
 * @returns {int}
 */
function SetMapAsPlayed();

/**
 * Print a hud message on all clients.
 *
 * @type {function}
 * @param {string} message
 */
function ShowMessage(message);

/**
 * Get the current server time
 *
 * @type {function}
 * @returns {float}
 */
function Time();

/**
 * Sweeps a hull along the specified line. Returns a CGameTrace with the trace result.
 *
 * @type {function}
 * @param {Vector} start
 * @param {Vector} end
 * @param {Vector} hullMin
 * @param {Vector} hullMax
 * @param {int} mask
 * @param {CBaseEntity|null} entToIgnore
 * @param {int} collisionGroup
 * @returns {CBaseEntity|null}
 */
function TraceHull(start, end, hullMin, hullMax, mask, entToIgnore, collisionGroup);

/**
 * Trace a line, then return the fraction along line that hits world or models.
 *
 * @type {function}
 * @param {Vector} start
 * @param {Vector} end
 * @param {CBaseEntity|null} entToIgnore
 * @returns {float}
 */
function TraceLine(start, end, entToIgnore);

/**
 * Given 2 points, ent to ignore (or array of ents to ignore), collision group and trace mask, returns a CGameTrace with the result.
 *
 * @type {function}
 * @param {Vector} start
 * @param {Vector} end
 * @param {int} mask
 * @param {object} ignore
 * @param {int} collisionGroup
 * @returns {CBaseEntity|null}
 */
function TraceLineEx(start, end, mask, ignore, collisionGroup);

/**
 * Trace a line, then return the fraction along line that hits world, models, players or npcs.
 *
 * @type {function}
 * @param {Vector} start
 * @param {Vector} end
 * @param {CBaseEntity|null} entToIgnore
 * @returns {float}
 */
function TraceLinePlayersIncluded(start, end, entToIgnore);

/**
 * Same as TraceLineEx, but will transform the trace based on any portals it passes through. If the last bool is true, it will transform based on the first portal it went though.
 *
 * @type {function}
 * @param {Vector} start
 * @param {Vector} end
 * @param {int} mask
 * @param {object} ignore
 * @param {int} collisionGroup
 * @param {bool} transformTrace
 * @returns {CBaseEntity|null}
 */
function TracePortalLine(start, end, mask, ignore, collisionGroup, transformTrace);

/**
 * Always returns true. Used in Portal 2 to test if DLC1 was installed.
 *
 * @type {function}
 * @returns {bool}
 */
function TryDLC1InstalledOrCatch();

/**
 * Generate a string guaranteed to be unique across the life of the script VM, with an optional root string. Useful for adding data to tables when not sure what keys are already in use in that table.
 *
 * @type {function}
 * @param {object} string
 * @returns {function}
 */
function UniqueString(string);

/**
 * Upgrades the paint gun of all players, if they are not holding one it will give them one.
 *
 * @type {function}
 */
function UpgradePlayerPaintgun();

/**
 * Upgrade the player's held gun to shoot both portals.
 *
 * @type {function}
 */
function UpgradePlayerPortalgun();

/**
 * Add Potatos to the player's held portal gun, and force it to be a dual device.
 *
 * @type {function}
 */
function UpgradePlayerPotatogun();


/*
 * =======================
 * CLASSES
 * =======================
 */

class StorageScope
{
    /**
     * Clear the specified key.
     *
     * @type {function}
     * @param {string} key
     */
    function Clear(key);

    /**
     * Clear all values in this scope.
     *
     * @type {function}
     */
    function ClearAll();

    /**
     * Gets the specified value as a float.
     *
     * @type {function}
     * @param {string} key
     * @returns {float}
     */
    function GetFloat(key);

    /**
     * Gets the specified value as an integer.
     *
     * @type {function}
     * @param {string} key
     * @returns {int}
     */
    function GetInt(key);

    /**
     * Gets the specified value.
     *
     * @type {function}
     * @param {string} key
     * @returns {string}
     */
    function GetString(key);

    /**
     * Gets the specified value as a vector.
     *
     * @type {function}
     * @param {string} key
     * @returns {Vector}
     */
    function GetVector(key);

    /**
     * Sets the specified value.
     *
     * @type {function}
     * @param {string} key
     * @param {float} value
     */
    function SetFloat(key, value);

    /**
     * Sets the specified value.
     *
     * @type {function}
     * @param {string} key
     * @param {int} value
     */
    function SetInt(key, value);

    /**
     * Sets the specified value.
     *
     * @type {function}
     * @param {string} key
     * @param {string} value
     */
    function SetString(key, value);

    /**
     * Sets the specified value.
     *
     * @type {function}
     * @param {string} key
     * @param {Vector} value
     */
    function SetVector(key, value);

}

class CLinkedPortalDoor extends CBaseAnimating
{
    /**
     * Get the instance handle of the door's linked partner.
     *
     * @type {function}
     * @returns {CBaseEntity|null}
     */
    function GetPartnerInstance();

    /**
     * Returns the name of the door's partner.
     *
     * @type {function}
     * @returns {string}
     */
    function GetPartnername();

}

class CBasePortal extends CBaseAnimating
{
    /**
     * Gets the half height of the portal.
     *
     * @type {function}
     * @returns {float}
     */
    function GetHalfHeight();

    /**
     * Gets the half width of the portal.
     *
     * @type {function}
     * @returns {float}
     */
    function GetHalfWidth();

    /**
     * Get the handle to the partner portal.
     *
     * @type {function}
     * @returns {CBaseEntity|null}
     */
    function GetPartner();

    /**
     * Gets the portal number. 1 for primary portal, 2 for secondary.
     *
     * @type {function}
     * @returns {int}
     */
    function GetPortalNumber();

    /**
     * Returns true if the portal is active, but not necessarily linked/open.
     *
     * @type {function}
     * @returns {bool}
     */
    function IsActive();

    /**
     * Returns true if this is a movable portal.
     *
     * @type {function}
     * @returns {bool}
     */
    function IsMobile();

    /**
     * Returns true if the portal is open and passable.
     *
     * @type {function}
     * @returns {bool}
     */
    function IsOpen();

}

class CEntities
{
    /**
     * Creates an entity by classname.
     *
     * @type {function}
     * @param {classname} className
     * @returns {CBaseEntity|null}
     */
    function CreateByClassname(className);

    /**
     * Calls the Spawn function for the specified entity.
     *
     * @type {function}
     * @param {CBaseEntity|null} ent
     */
    function DispatchSpawn(ent);

    /**
     * Find entities by class name. Pass `null` to start an iteration, or reference to a previously found entity to continue a search.
     *
     * @type {function}
     * @param {CBaseEntity|null} previous
     * @param {classname} className
     * @returns {CBaseEntity|null}
     */
    function FindByClassname(previous, className);

    /**
     * Find entities by class name nearest to a point.
     *
     * @type {function}
     * @param {classname} className
     * @param {Vector} position
     * @param {float} radius
     * @returns {CBaseEntity|null}
     */
    function FindByClassnameNearest(className, position, radius);

    /**
     * Find entities by class name within a radius. Pass `null` to start an iteration, or reference to a previously found entity to continue a search.
     *
     * @type {function}
     * @param {CBaseEntity|null} previous
     * @param {classname} className
     * @param {Vector} position
     * @param {float} radius
     * @returns {CBaseEntity|null}
     */
    function FindByClassnameWithin(previous, className, position, radius);

    /**
     * Find entities by model name. Pass `null` to start an iteration, or reference to a previously found entity to continue a search.
     *
     * @type {function}
     * @param {CBaseEntity|null} previous
     * @param {string} modelName
     * @returns {CBaseEntity|null}
     */
    function FindByModel(previous, modelName);

    /**
     * Find entities by name. Pass `null` to start an iteration, or reference to a previously found entity to continue a search.
     *
     * @type {function}
     * @param {CBaseEntity|null} previous
     * @param {string} targetName
     * @returns {CBaseEntity|null}
     */
    function FindByName(previous, targetName);

    /**
     * Find entities by name nearest to a point.
     *
     * @type {function}
     * @param {string} targetName
     * @param {Vector} position
     * @param {float} radius
     * @returns {CBaseEntity|null}
     */
    function FindByNameNearest(targetName, position, radius);

    /**
     * Find entities by name within a radius. Pass `null` to start an iteration, or reference to a previously found entity to continue a search.
     *
     * @type {function}
     * @param {CBaseEntity|null} previous
     * @param {string} targetName
     * @param {Vector} position
     * @param {float} radius
     * @returns {CBaseEntity|null}
     */
    function FindByNameWithin(previous, targetName, position, radius);

    /**
     * Find entities by which target the specified name. Pass `null` to start an iteration, or reference to a previously found entity to continue a search.
     *
     * @type {function}
     * @param {CBaseEntity|null} previous
     * @param {string} target
     * @returns {CBaseEntity|null}
     */
    function FindByTarget(previous, target);

    /**
     * Find entities within a radius. Pass `null` to start an iteration, or reference to a previously found entity to continue a search.
     *
     * @type {function}
     * @param {CBaseEntity|null} previous
     * @param {Vector} center
     * @param {float} radius
     * @returns {CBaseEntity|null}
     */
    function FindInSphere(previous, center, radius);

    /**
     * Begin an iteration over the list of entities.
     *
     * @type {function}
     * @returns {CBaseEntity|null}
     */
    function First();

    /**
     * Returns handle to entity based on its ent index. The index is 1-based.
     *
     * @type {function}
     * @param {int} index
     * @returns {CBaseEntity|null}
     */
    function GetByIndex(index);

    /**
     * Continue an iteration over the list of entities, providing reference to a previously found entity.
     *
     * @type {function}
     * @param {CBaseEntity|null} previous
     * @returns {CBaseEntity|null}
     */
    function Next(previous);

}

class CBaseFlex extends CBaseAnimating
{
    /**
     * Returns the instance of the oldest active scene entity (if any).
     *
     * @type {function}
     * @returns {CBaseEntity|null}
     */
    function GetCurrentScene();

    /**
     * Returns the instance of the scene entity at the specified index.
     *
     * @type {function}
     * @param {int} index
     * @returns {CBaseEntity|null}
     */
    function GetSceneByIndex(index);

}

class CBasePlayer extends CBaseFlex
{
    /**
     * Clears the active weapon
     *
     * @type {function}
     */
    function ClearActiveWeapon();

    /**
     * Gets the active weapon for the player.
     *
     * @type {function}
     * @returns {CBaseEntity|null}
     */
    function GetActiveWeapon();

    /**
     * Gets the ammo count for the specified type.
     *
     * @type {function}
     * @param {int} type
     * @returns {int}
     */
    function GetAmmoCount(type);

    /**
     * Returns the button bitfield for the player.
     *
     * @type {function}
     * @returns {int}
     */
    function GetButtons();

    /**
     * Returns the player's name.
     *
     * @type {function}
     * @returns {string}
     */
    function GetPlayerName();

    /**
     * Get the vehicle the player is in, or null if the player is not in one
     *
     * @type {function}
     * @returns {CBaseEntity|null}
     */
    function GetVehicle();

    /**
     * Gets a weapon by index on the player.
     *
     * @type {function}
     * @param {int} index
     * @returns {CBaseEntity|null}
     */
    function GetWeapon(index);

    /**
     * Gets the max number of weapons the player can carry.
     *
     * @type {function}
     * @returns {int}
     */
    function GetWeaponCount();

    /**
     * Returns true if the player is dead
     *
     * @type {function}
     * @returns {bool}
     */
    function IsDead();

    /**
     * Returns true if the player is in a vehicle
     *
     * @type {function}
     * @returns {bool}
     */
    function IsInAVehicle();

    /**
     * Returns true if the player is in noclip mode.
     *
     * @type {function}
     * @returns {bool}
     */
    function IsNoclipping();

    /**
     * Returns true if the player is underwater
     *
     * @type {function}
     * @returns {bool}
     */
    function IsPlayerUnderwater();

    /**
     * Sets the current active weapon for the player.
     *
     * @type {function}
     * @param {CBaseEntity|null} weapon
     */
    function SetActiveWeapon(weapon);

    /**
     * Sets the player ammo count.
     *
     * @type {function}
     * @param {int} type
     * @param {int} count
     */
    function SetAmmoCount(type, count);

}

class CPanoramaScreen extends CBaseEntity
{
    /**
     * Adds the CSS class
     *
     * @type {function}
     * @param {classname} className
     */
    function AddCSSClass(className);

    /**
     * Removes the CSS class
     *
     * @type {function}
     * @param {classname} className
     */
    function RemoveCSSClass(className);

    /**
     * Executes JavaScript in the panel scope. Example: `Test2.TestFunction()`.
     *
     * @type {function}
     * @param {string} script
     */
    function RunJSScript(script);

    /**
     * Sets the screen to be active or inactive.
     *
     * @type {function}
     * @param {bool} active
     */
    function SetActive(active);

}

class CBaseCombatWeapon extends CBaseAnimating
{
    /**
     * Drop the weapon on the ground with the specified velocity
     *
     * @type {function}
     * @param {Vector} velocity
     */
    function Drop(velocity);

    /**
     * Get current ammo in clip 1.
     *
     * @type {function}
     * @returns {int}
     */
    function GetClip1();

    /**
     * Get current ammo in clip 2.
     *
     * @type {function}
     * @returns {int}
     */
    function GetClip2();

    /**
     * Returns the fire rate for this weapon.
     *
     * @type {function}
     * @returns {float}
     */
    function GetFireRate();

    /**
     * Get the max ammo in clip 1.
     *
     * @type {function}
     * @returns {int}
     */
    function GetMaxClip1();

    /**
     * Get the max ammo in clip 2.
     *
     * @type {function}
     * @returns {int}
     */
    function GetMaxClip2();

    /**
     * Get the weapon name.
     *
     * @type {function}
     * @returns {string}
     */
    function GetName();

    /**
     * Get the primary ammo count.
     *
     * @type {function}
     * @returns {int}
     */
    function GetPrimaryAmmoCount();

    /**
     * Get the primary ammo type.
     *
     * @type {function}
     * @returns {int}
     */
    function GetPrimaryAmmoType();

    /**
     * Get the weapon's display name.
     *
     * @type {function}
     * @returns {string}
     */
    function GetPrintName();

    /**
     * Get the secondary ammo count.
     *
     * @type {function}
     * @returns {int}
     */
    function GetSecondaryAmmoCount();

    /**
     * Get the secondary ammo type.
     *
     * @type {function}
     * @returns {int}
     */
    function GetSecondaryAmmoType();

    /**
     * Lowers the weapon
     *
     * @type {function}
     * @returns {bool}
     */
    function Lower();

    /**
     * Readies the weapon
     *
     * @type {function}
     * @returns {bool}
     */
    function Ready();

    /**
     * Returns if this weapon uses clips for ammo 1.
     *
     * @type {function}
     * @returns {bool}
     */
    function UsesClipsForAmmo1();

    /**
     * Returns if this weapon uses clips for ammo 2.
     *
     * @type {function}
     * @returns {bool}
     */
    function UsesClipsForAmmo2();

}

class CFuncTrackTrain extends CBaseEntity
{
    /**
     * Get a position on the track x seconds in the future
     *
     * @type {function}
     * @param {float} delay
     * @param {float} speed
     * @returns {Vector}
     */
    function GetFuturePosition(delay, speed);

}

class ScriptStorageMgr
{
    /**
     * Creates a new named storage scope
     *
     * @type {function}
     * @param {string} name
     * @returns {CBaseEntity|null}
     */
    function CreateScope(name);

}

class CPortal_Player extends CBaseMultiplayerPlayer
{
    /**
     * Get number of wheatley monitors destroyed by the player.
     *
     * @type {function}
     * @returns {int}
     */
    function GetWheatleyMonitorDestructionCount();

    /**
     * Set number of wheatley monitors destroyed by the player.
     *
     * @type {function}
     */
    function IncWheatleyMonitorDestructionCount();

    /**
     * Turns Off the Potatos material light
     *
     * @type {function}
     */
    function TurnOffPotatos();

    /**
     * Turns On the Potatos material light
     *
     * @type {function}
     */
    function TurnOnPotatos();

}

class CPointViewControl extends CBaseEntity
{
    /**
     * Get camera's current fov setting as an integer.
     *
     * @type {function}
     * @returns {int}
     */
    function GetFov();

    /**
     * Change the camera's FOV over time.
     *
     * @type {function}
     * @param {int} fov
     * @param {float} rate
     */
    function SetFov(fov, rate);

}

class CWeaponPaintGun extends CBaseCombatWeapon
{
    /**
     * Activates the specified paint power on the gun.
     *
     * @type {function}
     * @param {int} paintType
     */
    function ActivatePaint(paintType);

    /**
     * Cycles to the next or previous paint power. Arg1: Whether to cycle forwards or backwards.
     *
     * @type {function}
     * @param {bool} forward
     */
    function CyclePaintPower(forward);

    /**
     * Deactivates all paint powers on the gun.
     *
     * @type {function}
     */
    function DeactivateAllPaints();

    /**
     * Deactivates the specified paint power.
     *
     * @type {function}
     * @param {int} paintType
     */
    function DeactivatePaint(paintType);

    /**
     * Returns the current active paint power.
     *
     * @type {function}
     * @returns {int}
     */
    function GetCurrentPaint();

    /**
     * Returns the number of paint powers the gun has access to.
     *
     * @type {function}
     * @returns {int}
     */
    function GetPaintCount();

    /**
     * Whether or not the gun has any paint powers at all, excluding NO_POWER
     *
     * @type {function}
     * @returns {bool}
     */
    function HasAnyPaintPower();

    /**
     * Whether or not the gun has the specified paint power.
     *
     * @type {function}
     * @param {int} paintType
     * @returns {bool}
     */
    function HasPaintPower(paintType);

    /**
     * Sets the gun's current power to the specified paint power.
     *
     * @type {function}
     * @param {int} paintType
     */
    function SetCurrentPaint(paintType);

}

class CPropPhysicsPaintable extends CBaseAnimating
{
    /**
     * Get the current paint type applied to the prop. Returns a value from the *_POWER enum.
     *
     * @type {function}
     * @returns {int}
     */
    function GetPaint();

    /**
     * Get the skin used when the prop is painted.
     *
     * @type {function}
     * @returns {int}
     */
    function GetPaintedSkin();

    /**
     * Get the skin used when the prop is not painted.
     *
     * @type {function}
     * @returns {int}
     */
    function GetUnPaintedSkin();

}

class CSceneManager extends CBaseEntity
{
    /**
     * Returns whether this actor is involved in a choreo scene.
     *
     * @type {function}
     * @param {CBaseEntity|null} actor
     * @returns {bool}
     */
    function IsSceneRunning(actor);

}

class CSceneEntity extends CBaseEntity
{
    /**
     * Adds a team (by index) to the broadcast list
     *
     * @type {function}
     * @param {int} team
     */
    function AddBroadcastTeamTarget(team);

    /**
     * Returns length of this scene in seconds.
     *
     * @type {function}
     * @returns {float}
     */
    function EstimateLength();

    /**
     * given an entity reference, such as !target, get actual entity from scene object
     *
     * @type {function}
     * @param {string} name
     * @returns {CBaseEntity|null}
     */
    function FindNamedEntity(name);

    /**
     * If this scene is currently paused.
     *
     * @type {function}
     * @returns {bool}
     */
    function IsPaused();

    /**
     * If this scene is currently playing.
     *
     * @type {function}
     * @returns {bool}
     */
    function IsPlayingBack();

    /**
     * given a dummy scene name and a vcd string, load the scene
     *
     * @type {function}
     * @param {string} filename
     * @param {string} sceneData
     * @returns {bool}
     */
    function LoadSceneFromString(filename, sceneData);

    /**
     * Removes a team (by index) from the broadcast list
     *
     * @type {function}
     * @param {int} team
     */
    function RemoveBroadcastTeamTarget(team);

}

class CBaseAnimating extends CBaseEntity
{
    /**
     * Find a bodygroup given its name, -1 if the bodygroup does not exist.
     *
     * @type {function}
     * @param {string} name
     * @returns {int}
     */
    function FindBodygroupByName(name);

    /**
     * Get world angles as a p,y,r vector for the specified attachment id.
     *
     * @type {function}
     * @param {int} attachIndex
     * @returns {Vector}
     */
    function GetAttachmentAngles(attachIndex);

    /**
     * Get world position for the specified attachement id.
     *
     * @type {function}
     * @param {int} attachIndex
     * @returns {Vector}
     */
    function GetAttachmentOrigin(attachIndex);

    /**
     * Gets the current part of a bodygroup group.
     *
     * @type {function}
     * @param {int} group
     * @returns {int}
     */
    function GetBodygroup(group);

    /**
     * Gets the number of parts in a bodygroup group.
     *
     * @type {function}
     * @param {int} group
     * @returns {int}
     */
    function GetBodygroupCount(group);

    /**
     * Returns the name of the bodygroup.
     *
     * @type {function}
     * @param {int} group
     * @returns {string}
     */
    function GetBodygroupName(group);

    /**
     * Returns the bodygroup part name.
     *
     * @type {function}
     * @param {int} group
     * @param {int} part
     * @returns {string}
     */
    function GetBodygroupPartName(group, part);

    /**
     * Returns the number of bones.
     *
     * @type {function}
     * @returns {int}
     */
    function GetBoneCount();

    /**
     * Returns the world origin of the bone.
     *
     * @type {function}
     * @param {int} bone
     * @returns {Vector}
     */
    function GetBonePosition(bone);

    /**
     * Returns the world rotation of the bone.
     *
     * @type {function}
     * @param {int} bone
     * @returns {Vector}
     */
    function GetBoneRotation(bone);

    /**
     * Returns the number of bodygroup groups.
     *
     * @type {function}
     * @returns {int}
     */
    function GetNumBodyGroups();

    /**
     * The scale size of the entity.
     *
     * @type {function}
     * @returns {int}
     */
    function GetObjectScaleLevel();

    /**
     * Returns the current playback rate
     *
     * @type {function}
     * @returns {float}
     */
    function GetPlaybackRate();

    /**
     * Returns pose parameter value based on index.
     *
     * @type {function}
     * @param {int} parameter
     * @returns {float}
     */
    function GetPoseParameter(parameter);

    /**
     * Returns the max value of the pose parameter.
     *
     * @type {function}
     * @param {int} parameter
     * @returns {float}
     */
    function GetPoseParameterMax(parameter);

    /**
     * Returns the min value of the pose parameter.
     *
     * @type {function}
     * @param {int} parameter
     * @returns {float}
     */
    function GetPoseParameterMin(parameter);

    /**
     * Returns the current sequence.
     *
     * @type {function}
     * @returns {int}
     */
    function GetSequence();

    /**
     * Returns the name of the sequence's activity.
     *
     * @type {function}
     * @param {int} sequence
     * @returns {string}
     */
    function GetSequenceActivityName(sequence);

    /**
     * Returns the number of available sequences.
     *
     * @type {function}
     * @returns {int}
     */
    function GetSequenceCount();

    /**
     * Gets the sequence cycle rate for the specified sequence.
     *
     * @type {function}
     * @param {int} sequence
     * @returns {float}
     */
    function GetSequenceCycleRate(sequence);

    /**
     * Gets the sequence duration for the specified sequence.
     *
     * @type {function}
     * @param {int} sequence
     * @returns {float}
     */
    function GetSequenceDuration(sequence);

    /**
     * Returns the name of the sequence, if it's valid.
     *
     * @type {function}
     * @param {int} sequence
     * @returns {string}
     */
    function GetSequenceName(sequence);

    /**
     * Gets the current model skin index.
     *
     * @type {function}
     * @returns {int}
     */
    function GetSkin();

    /**
     * Is the current activity finished?
     *
     * @type {function}
     * @returns {bool}
     */
    function IsActivityFinished();

    /**
     * Is the current sequence finished?
     *
     * @type {function}
     * @returns {bool}
     */
    function IsSequenceFinished();

    /**
     * Returns if the specified sequence is looped or not.
     *
     * @type {function}
     * @param {int} sequence
     * @returns {bool}
     */
    function IsSequenceLooped(sequence);

    /**
     * Checks if the specified sequence is valid.
     *
     * @type {function}
     * @param {int} sequence
     * @returns {bool}
     */
    function IsValidSequence(sequence);

    /**
     * Looks up an activity based on name.
     *
     * @type {function}
     * @param {string} activity
     * @returns {int}
     */
    function LookupActivity(activity);

    /**
     * Get the named attachement id, or -1 if not found.
     *
     * @type {function}
     * @param {string} attachment
     * @returns {int}
     */
    function LookupAttachment(attachment);

    /**
     * Lookup a pose parameter based on its name. Returns -1 if not found
     *
     * @type {function}
     * @param {string} parameter
     * @returns {int}
     */
    function LookupPoseParameter(parameter);

    /**
     * Changes a bodygroup group to a different part.
     *
     * @type {function}
     * @param {int} group
     * @param {int} value
     */
    function SetBodygroup(group, value);

    /**
     * Sets the current playback rate
     *
     * @type {function}
     * @param {float} playbackRate
     */
    function SetPlaybackRate(playbackRate);

    /**
     * Set pose parameter value based on index.
     *
     * @type {function}
     * @param {int} parameter
     * @param {float} value
     * @returns {float}
     */
    function SetPoseParameter(parameter, value);

    /**
     * Sets the current sequence.
     *
     * @type {function}
     * @param {int} sequence
     */
    function SetSequence(sequence);

    /**
     * Sets the current model skin index.
     *
     * @type {function}
     * @param {int} skin
     */
    function SetSkin(skin);

    /**
     * Looks up a sequence based on name.
     *
     * @type {function}
     * @param {string} LookupSequence
     * @returns {int}
     */
    function sequence(LookupSequence);

}

class CPropLinkedPortalDoor extends CBaseAnimating
{
    /**
     * Get the instance handle of the door's linked partner.
     *
     * @type {function}
     * @returns {CBaseEntity|null}
     */
    function GetPartnerInstance();

    /**
     * Returns the name of the door's partner.
     *
     * @type {function}
     * @returns {string}
     */
    function GetPartnername();

}

class CPropWeightedCube extends CBaseAnimating
{
    /**
     * Get the behavior type of the cube. Returns a value from the CUBE_BEHAVIOR_* enum.
     *
     * @type {function}
     * @returns {int}
     */
    function GetCubeBehavior();

    /**
     * Get the shape of the cube (IE what buttons it presses). This is a number which matches the CUBE_SHAPE_* enum, or other values if a custom shape was set.
     *
     * @type {function}
     * @returns {int}
     */
    function GetCubeShape();

    /**
     * Get the instance handle of the invisible env_portal_laser outputting from this cube, or null if not emitting.
     *
     * @type {function}
     * @returns {CBaseEntity|null}
     */
    function GetLaser();

    /**
     * Get the current paint type applied to the cube. Returns a value from the *_POWER enum.
     *
     * @type {function}
     * @returns {int}
     */
    function GetPaint();

    /**
     * Get the instance handle of the schrodinger's partner.
     *
     * @type {function}
     * @returns {CBaseEntity|null}
     */
    function GetPartner();

    /**
     * Check whether the laser output is locked on.
     *
     * @type {function}
     * @returns {bool}
     */
    function IsEmissionLocked();

    /**
     * Check whether the cube is 'activated', pressing a button.
     *
     * @type {function}
     * @returns {bool}
     */
    function IsPressingButton();

}

class CBaseEntity
{
    /**
     * Adds an I/O connection that will call the named function in this entity's scope when the specified output fires.
     *
     * @type {function}
     * @param {string} output
     * @param {string} funcName
     */
    function ConnectOutput(output, funcName);

    /**
     * Kill this entity.
     *
     * @type {function}
     */
    function Destroy();

    /**
     * Removes the output created by ConnectOutput().
     *
     * @type {function}
     * @param {string} output
     * @param {string} funcName
     */
    function DisconnectOutput(output, funcName);

    /**
     * Plays a sound from this entity.
     *
     * @type {function}
     * @param {string} soundName
     */
    function EmitSound(soundName);

    /**
     * Get eye pitch, yaw, roll as a vector.
     *
     * @type {function}
     * @returns {Vector}
     */
    function EyeAngles();

    /**
     * Get eye local pitch, yaw, roll as a vector.
     *
     * @type {function}
     * @returns {Vector}
     */
    function EyeLocalAngles();

    /**
     * Get vector to eye position - absolute coords.
     *
     * @type {function}
     * @returns {Vector}
     */
    function EyePosition();

    /**
     * Returns an arbitary 'first' child for this entity, or null if this entity has no children. Use NextMovePeer() to iterate through children.
     *
     * @type {function}
     * @returns {CBaseEntity|null}
     */
    function FirstMoveChild();

    /**
     * Get entity pitch, yaw, roll as a vector.
     *
     * @type {function}
     * @returns {Vector}
     */
    function GetAngles();

    /**
     * Get the local angular velocity - returns a vector of pitch,yaw,roll.
     *
     * @type {function}
     * @returns {Vector}
     */
    function GetAngularVelocity();

    /**
     * Get a vector containing max bounds in local scope.
     *
     * @type {function}
     * @returns {Vector}
     */
    function GetBoundingMaxs();

    /**
     * Get a vector containing max bounds, centered on object, taking the object's orientation into account.
     *
     * @type {function}
     * @returns {Vector}
     */
    function GetBoundingMaxsOriented();

    /**
     * Get a vector containing min bounds in local scape.
     *
     * @type {function}
     * @returns {Vector}
     */
    function GetBoundingMins();

    /**
     * Get a vector containing min bounds, centered on object, taking the object's orientation into account.
     *
     * @type {function}
     * @returns {Vector}
     */
    function GetBoundingMinsOriented();

    /**
     * Get vector to center of object - absolute coords.
     *
     * @type {function}
     * @returns {Vector}
     */
    function GetCenter();

    /**
     * Get the classname for this entity.
     *
     * @type {function}
     * @returns {string}
     */
    function GetClassname();

    /**
     * Get the collision group, which will be a `COLLISION_GROUP_*` constant.
     *
     * @type {function}
     * @returns {int}
     */
    function GetCollisionGroup();

    /**
     * Get the current elasticity value for this entity.
     *
     * @type {function}
     * @returns {float}
     */
    function GetElasticity();

    /**
     * Get the forward vector of the entity.
     *
     * @type {function}
     * @returns {Vector}
     */
    function GetForwardVector();

    /**
     * Get the current friction for this entity.
     *
     * @type {function}
     * @returns {float}
     */
    function GetFriction();

    /**
     * Get the current gravity for this entity, vertically.
     *
     * @type {function}
     * @returns {float}
     */
    function GetGravity();

    /**
     * Return the current health of this entity.
     *
     * @type {function}
     * @returns {int}
     */
    function GetHealth();

    /**
     * Get a KeyValue on this entity as a bool.
     *
     * @type {function}
     * @param {string} name
     * @returns {bool}
     */
    function GetKeyValueBool(name);

    /**
     * Get a KeyValue on this entity as a float.
     *
     * @type {function}
     * @param {string} name
     * @returns {float}
     */
    function GetKeyValueFloat(name);

    /**
     * Get a KeyValue on this entity as an integer.
     *
     * @type {function}
     * @param {string} name
     * @returns {int}
     */
    function GetKeyValueInt(name);

    /**
     * Get a KeyValue on this entity as a string.
     *
     * @type {function}
     * @param {string} name
     * @returns {string}
     */
    function GetKeyValueString(name);

    /**
     * Get the left vector of the entity.
     *
     * @type {function}
     * @returns {Vector}
     */
    function GetLeftVector();

    /**
     * Return the current maximum health of this entity.
     *
     * @type {function}
     * @returns {int}
     */
    function GetMaxHealth();

    /**
     * Returns access to the $keyvalues definition for this entity's model.
     *
     * @type {function}
     * @returns {CBaseEntity|null}
     */
    function GetModelKeyValues();

    /**
     * Returns the name of the model this entity is set to. For brush entities, this will be in the form "123".
     *
     * @type {function}
     * @returns {string}
     */
    function GetModelName();

    /**
     * Returns the current move collision mode, which will be a `MOVECOLLIDE_*` constant.
     *
     * @type {function}
     * @returns {int}
     */
    function GetMoveCollide();

    /**
     * If in hierarchy, retrieves the entity's parent.
     *
     * @type {function}
     * @returns {CBaseEntity|null}
     */
    function GetMoveParent();

    /**
     * Returns the current move type, which will be a `MOVETYPE_*` constant.
     *
     * @type {function}
     * @returns {int}
     */
    function GetMoveType();

    /**
     * Get the targetname of this entity.
     *
     * @type {function}
     * @returns {string}
     */
    function GetName();

    /**
     * Get the namespace for this entity's classname.
     *
     * @type {function}
     * @returns {string}
     */
    function GetNamespace();

    /**
     * Get the absolute position of this entity.
     *
     * @type {function}
     * @returns {Vector}
     */
    function GetOrigin();

    /**
     * Gets this entity's owner.
     *
     * @type {function}
     * @returns {CBaseEntity|null}
     */
    function GetOwner();

    /**
     * Get the targetname stripped of template unique decoration like name&0123.
     *
     * @type {function}
     * @returns {string}
     */
    function GetPreTemplateName();

    /**
     * If in hierarchy, walks up the hierarchy to find the root parent.
     *
     * @type {function}
     * @returns {CBaseEntity|null}
     */
    function GetRootMoveParent();

    /**
     * Retrieve the unique identifier used to refer to the entity within the scripting system.
     *
     * @type {function}
     * @returns {string}
     */
    function GetScriptId();

    /**
     * Retrieve the script-side data associated with an entity, or null if not created.
     *
     * @type {function}
     * @returns {CBaseEntity|null}
     */
    function GetScriptScope();

    /**
     * Returns float duration of the sound. The optional actor model is used to resolve $gender variables.
     *
     * @type {function}
     * @param {string} soundName
     * @param {string} actorModel
     * @returns {float}
     */
    function GetSoundDuration(soundName, actorModel);

    /**
     * Get this entity's team number.
     *
     * @type {function}
     * @returns {int}
     */
    function GetTeam();

    /**
     * Get the up vector of the entity.
     *
     * @type {function}
     * @returns {Vector}
     */
    function GetUpVector();

    /**
     * Return this entity's absolute linear velocity.
     *
     * @type {function}
     * @returns {Vector}
     */
    function GetVelocity();

    /**
     * Returns if this entity is a BSP model or not (i.e. func_brush).
     *
     * @type {function}
     * @returns {bool}
     */
    function IsBSPModel();

    /**
     * Returns if this entity is floating in water or not.
     *
     * @type {function}
     * @returns {bool}
     */
    function IsFloating();

    /**
     * Returns the 'next' sibling for this entity, or null if all siblings were returned. Calling this repeatedly on FirstMoveChild() will give all children in turn.
     *
     * @type {function}
     * @returns {CBaseEntity|null}
     */
    function NextMovePeer();

    /**
     * Ensure this model is loaded for later use. Should be called inside the Precache() method.
     *
     * @type {function}
     * @param {string} modelName
     */
    function PrecacheModel(modelName);

    /**
     * Ensure this sound is loaded for later use. Should be called inside the Precache() method.
     *
     * @type {function}
     * @param {string} sound
     */
    function PrecacheScriptSound(sound);

    /**
     * Precache a game sound or raw sound for later playing.
     *
     * @type {function}
     * @param {string} sound
     */
    function PrecacheSoundScript(sound);

    /**
     * Set the absolute position of this entity.
     *
     * @type {function}
     * @param {Vector} origin
     */
    function SetAbsOrigin(origin);

    /**
     * Set entity pitch, yaw, roll.
     *
     * @type {function}
     * @param {float} pitch
     * @param {float} roll
     * @param {float} yaw
     */
    function SetAngles(pitch, roll, yaw);

    /**
     * Set the local angular velocity. The move type must be set for this to have effect.
     *
     * @type {function}
     * @param {float} pitch
     * @param {float} roll
     * @param {float} yaw
     */
    function SetAngularVelocity(pitch, roll, yaw);

    /**
     * Sets the collision group to one of the `COLLISION_GROUP_*` constants.
     *
     * @type {function}
     * @param {int} group
     */
    function SetCollisionGroup(group);

    /**
     * Set the elasticity value for this entity.
     *
     * @type {function}
     * @param {float} elasticity
     */
    function SetElasticity(elasticity);

    /**
     * Set the orientation of the entity to have this forward vector.
     *
     * @type {function}
     * @param {Vector} forward
     */
    function SetForwardVector(forward);

    /**
     * Set the friction for this entity.
     *
     * @type {function}
     * @param {float} friction
     */
    function SetFriction(friction);

    /**
     * Sets gravity on this entity. Only affects gravity along Z axis
     *
     * @type {function}
     * @param {float} gravity
     */
    function SetGravity(gravity);

    /**
     * Set the health for this entity. Zero will cause instant death.
     *
     * @type {function}
     * @param {int} health
     */
    function SetHealth(health);

    /**
     * Set the maximum health for this entity.
     *
     * @type {function}
     * @param {int} maxhealth
     */
    function SetMaxHealth(maxhealth);

    /**
     * Change the model used for the entity. The model must be precached manually.
     *
     * @type {function}
     * @param {string} modelName
     */
    function SetModel(modelName);

    /**
     * Set the move collision mode to one of the `MOVECOLLIDE_*` constants, determining how this entity reacts to collisions.
     *
     * @type {function}
     * @param {int} moveCollide
     */
    function SetMoveCollide(moveCollide);

    /**
     * Set the move type to one of the `MOVETYPE_*` constants, which determines how this entity is moved by velocity values.
     *
     * @type {function}
     * @param {int} moveType
     */
    function SetMoveType(moveType);

    /**
     * Teleport the entity to the specified position.
     *
     * @type {function}
     * @param {Vector} origin
     */
    function SetOrigin(origin);

    /**
     * Set this entity's owner. Owners are used for things like projectiles.
     *
     * @type {function}
     * @param {CBaseEntity|null} owner
     */
    function SetOwner(owner);

    /**
     * Sets the parent entity.
     *
     * @type {function}
     * @param {CBaseEntity|null} parent
     */
    function SetParent(parent);

    /**
     * Sets the parent entity with an attachment index.
     *
     * @type {function}
     * @param {CBaseEntity|null} parent
     * @param {int} attachmentIndex
     */
    function SetParentWithAttachment(parent, attachmentIndex);

    /**
     * Set the bounding box size for this entity.
     *
     * @type {function}
     * @param {Vector} mins
     * @param {Vector} maxes
     */
    function SetSize(mins, maxes);

    /**
     * Assign this entity to a different team.
     *
     * @type {function}
     * @param {int} team
     */
    function SetTeam(team);

    /**
     * Set this entity's absolute linear velocity. The move type must be set for this to have effect.
     *
     * @type {function}
     * @param {Vector} velocityVector
     */
    function SetVelocity(velocityVector);

    /**
     * Spawns the entity
     *
     * @type {function}
     */
    function Spawn();

    /**
     * Stops a sound on this entity.
     *
     * @type {function}
     * @param {string} soundName
     */
    function StopSound(soundName);

    /**
     * Teleport the entity to a new position with angles.
     *
     * @type {function}
     * @param {Vector} origin
     * @param {Vector} angles
     */
    function Teleport(origin, angles);

    /**
     * Ensure that an entity's script scope has been created.
     *
     * @type {function}
     * @returns {bool}
     */
    function ValidateScriptScope();

    /**
     * Returns the index for this entity. This is unique among live entities, but could change during save/load.
     *
     * @type {function}
     * @returns {int}
     */
    function entindex();

}

class CPropPortal extends CBasePortal
{
    /**
     * Fizzle the portal
     *
     * @type {function}
     */
    function Fizzle();

    /**
     * Returns the handle to the player who fired the portal, or null if none
     *
     * @type {function}
     * @returns {CBaseEntity|null}
     */
    function GetFiredByPlayer();

    /**
     * Gets the portal's linkage ID
     *
     * @type {function}
     * @returns {int}
     */
    function GetLinkageGroupID();

    /**
     * Place the portal at a new location
     *
     * @type {function}
     * @param {Vector} origin
     * @param {Vector} angles
     */
    function NewLocation(origin, angles);

    /**
     * Resize the portal. Parameters are half width and half height, respectively
     *
     * @type {function}
     * @param {float} halfWidth
     * @param {float} halfHeight
     */
    function Resize(halfWidth, halfHeight);

    /**
     * Activates or deactivates a portal
     *
     * @type {function}
     * @param {bool} state
     */
    function SetActivatedState(state);

    /**
     * Sets the portal's linkage ID
     *
     * @type {function}
     * @param {int} groupID
     */
    function SetLinkageGroupID(groupID);

}

class CScriptKeyValues
{
    /**
     * Clears this KeyValues object.
     *
     * @type {function}
     */
    function Clear();

    /**
     * Dump the object to console.
     *
     * @type {function}
     */
    function Dump();

    /**
     * Find a child KeyValues object associated with the specified key name. If create is true a new KeyValues object is created if no matching key exists, otherwise null is returned.
     *
     * @type {function}
     * @param {string} name
     * @param {bool} create
     * @returns {CBaseEntity|null}
     */
    function FindKey(name, create);

    /**
     * Return the first sub key object.
     *
     * @type {function}
     * @returns {CBaseEntity|null}
     */
    function GetFirstSubKey();

    /**
     * Return the associated bool value for this key name.
     *
     * @type {function}
     * @param {string} name
     * @returns {bool}
     */
    function GetKeyBool(name);

    /**
     * Return the associated float value for this key name.
     *
     * @type {function}
     * @param {string} name
     * @returns {float}
     */
    function GetKeyFloat(name);

    /**
     * Return the associated integer value for this key name.
     *
     * @type {function}
     * @param {string} name
     * @returns {int}
     */
    function GetKeyInt(name);

    /**
     * Return the associated string value for this key name.
     *
     * @type {function}
     * @param {string} name
     * @returns {string}
     */
    function GetKeyString(name);

    /**
     * Return the next key object in a sub key group.
     *
     * @type {function}
     * @returns {CBaseEntity|null}
     */
    function GetNextKey();

    /**
     * Return true if key name has no value.
     *
     * @type {function}
     * @param {string} name
     * @returns {bool}
     */
    function IsKeyEmpty(name);

    /**
     * Delete the contents of this KeyValues object.
     *
     * @type {function}
     */
    function ReleaseKeyValues();

    /**
     * Sets the associated bool value for this key name.
     *
     * @type {function}
     * @param {string} name
     * @param {bool} value
     */
    function SetKeyBool(name, value);

    /**
     * Sets the associated float value for this key name.
     *
     * @type {function}
     * @param {string} name
     * @param {float} value
     */
    function SetKeyFloat(name, value);

    /**
     * Sets the associated integer value for this key name.
     *
     * @type {function}
     * @param {string} name
     * @param {int} value
     */
    function SetKeyInt(name, value);

    /**
     * Sets the associated string value for this key name.
     *
     * @type {function}
     * @param {string} name
     * @param {string} value
     */
    function SetKeyString(name, value);

}

class CTakeDamageInfo
{
    /**
     * Adds to the damage.
     *
     * @type {function}
     * @param {float} additional
     */
    function AddDamage(additional);

    /**
     * Adds to the damage type.
     *
     * @type {function}
     * @param {int} extraDamageType
     */
    function AddDamageType(extraDamageType);

    /**
     * Checks if the base damage is valid.
     *
     * @type {function}
     * @returns {bool}
     */
    function BaseDamageIsValid();

    /**
     * Gets the name of the ammo which caused damage. This can be an ammo name, classname for physics objects, or a model name if thrown by the Gravity Gun.
     *
     * @type {function}
     * @returns {string}
     */
    function GetAmmoName();

    /**
     * Gets the ammo type.
     *
     * @type {function}
     * @returns {int}
     */
    function GetAmmoType();

    /**
     * Gets the attacker, which is the ultimate cause of damage.
     *
     * @type {function}
     * @returns {CBaseEntity|null}
     */
    function GetAttacker();

    /**
     * Gets the base damage.
     *
     * @type {function}
     * @returns {float}
     */
    function GetBaseDamage();

    /**
     * Gets the damage.
     *
     * @type {function}
     * @returns {float}
     */
    function GetDamage();

    /**
     * Gets the damage custom.
     *
     * @type {function}
     * @returns {int}
     */
    function GetDamageCustom();

    /**
     * Gets the damage force.
     *
     * @type {function}
     * @returns {Vector}
     */
    function GetDamageForce();

    /**
     * Gets the damage position.
     *
     * @type {function}
     * @returns {Vector}
     */
    function GetDamagePosition();

    /**
     * Gets the damage stats.
     *
     * @type {function}
     * @returns {int}
     */
    function GetDamageStats();

    /**
     * Gets the damage type.
     *
     * @type {function}
     * @returns {int}
     */
    function GetDamageType();

    /**
     * Gets whether other players have been damaged.
     *
     * @type {function}
     * @returns {int}
     */
    function GetDamagedOtherPlayers();

    /**
     * Gets the inflictor, which is the direct cause of damage like a grenade or zombie.
     *
     * @type {function}
     * @returns {CBaseEntity|null}
     */
    function GetInflictor();

    /**
     * Gets the max damage.
     *
     * @type {function}
     * @returns {float}
     */
    function GetMaxDamage();

    /**
     * Gets the reported damage position.
     *
     * @type {function}
     * @returns {Vector}
     */
    function GetReportedPosition();

    /**
     * Gets the weapon.
     *
     * @type {function}
     * @returns {CBaseEntity|null}
     */
    function GetWeapon();

    /**
     * Scales the damage.
     *
     * @type {function}
     * @param {float} factor
     */
    function ScaleDamage(factor);

    /**
     * Scales the damage force.
     *
     * @type {function}
     * @param {float} factor
     */
    function ScaleDamageForce(factor);

    /**
     * Sets the ammo type.
     *
     * @type {function}
     * @param {int} ammoType
     */
    function SetAmmoType(ammoType);

    /**
     * Sets the attacker.
     *
     * @type {function}
     * @param {CBaseEntity|null} attacker
     */
    function SetAttacker(attacker);

    /**
     * Sets the damage.
     *
     * @type {function}
     * @param {float} damage
     */
    function SetDamage(damage);

    /**
     * Sets the damage custom.
     *
     * @type {function}
     * @param {int} custType
     */
    function SetDamageCustom(custType);

    /**
     * Sets the damage force.
     *
     * @type {function}
     * @param {Vector} force
     */
    function SetDamageForce(force);

    /**
     * Sets the damage position.
     *
     * @type {function}
     * @param {Vector} position
     */
    function SetDamagePosition(position);

    /**
     * Sets the damage stats.
     *
     * @type {function}
     * @param {int} stats
     */
    function SetDamageStats(stats);

    /**
     * Sets the damage type.
     *
     * @type {function}
     * @param {int} damageType
     */
    function SetDamageType(damageType);

    /**
     * Sets whether other players have been damaged.
     *
     * @type {function}
     * @param {int} count
     */
    function SetDamagedOtherPlayers(count);

    /**
     * Sets the inflictor.
     *
     * @type {function}
     * @param {CBaseEntity|null} inflictor
     */
    function SetInflictor(inflictor);

    /**
     * Sets the max damage.
     *
     * @type {function}
     * @param {float} maxDamage
     */
    function SetMaxDamage(maxDamage);

    /**
     * Sets the reported damage position.
     *
     * @type {function}
     * @param {Vector} position
     */
    function SetReportedPosition(position);

    /**
     * Sets the weapon.
     *
     * @type {function}
     * @param {CBaseEntity|null} weapon
     */
    function SetWeapon(weapon);

    /**
     * Removes from the damage.
     *
     * @type {function}
     * @param {float} remove
     */
    function SubtractDamage(remove);

}

class CPlayerVoiceListener
{
    /**
     * Returns the number of seconds the player has been continuously speaking.
     *
     * @type {function}
     * @param {int} playerIndex
     * @returns {float}
     */
    function GetPlayerSpeechDuration(playerIndex);

    /**
     * Returns whether the player specified is speaking.
     *
     * @type {function}
     * @param {int} playerIndex
     * @returns {bool}
     */
    function IsPlayerSpeaking(playerIndex);

}

class CPlaytestManager
{
    /**
     * Begins recording of playtest info
     *
     * @type {function}
     */
    function BeginPlaytest();

    /**
     * Ends recording of playtest info
     *
     * @type {function}
     */
    function EndPlaytest();

}

class CBaseFilter extends CBaseEntity
{
    /**
     * Check if the given caller and damage info pass the damage filter. The caller is the one who requests the filter result; For example, the entity being damaged when using this as a damage filter.
     *
     * @type {function}
     * @param {CBaseEntity|null} caller
     * @param {CTakeDamageInfo} info
     * @returns {bool}
     */
    function PassesDamageFilter(caller, info);

    /**
     * Check if the given caller and entity pass the filter. The caller is the one who requests the filter result; For example, the entity being damaged when using this as a damage filter.
     *
     * @type {function}
     * @param {CBaseEntity|null} caller
     * @param {CBaseEntity|null} target
     * @returns {bool}
     */
    function PassesFilter(caller, target);

}

class CGameTrace
{
    /**
     * Returns true if the trace hit anything
     *
     * @type {function}
     * @returns {bool}
     */
    function DidHit();

    /**
     * Returns true if the trace hit non-world entity
     *
     * @type {function}
     * @returns {bool}
     */
    function DidHitNonWorldEntity();

    /**
     * Returns true if trace hit world
     *
     * @type {function}
     * @returns {bool}
     */
    function DidHitWorld();

    /**
     * Returns the contents flags of the hit entity or surface
     *
     * @type {function}
     * @returns {int}
     */
    function GetContents();

    /**
     * Returns the end position of the trace
     *
     * @type {function}
     * @returns {Vector}
     */
    function GetEndPos();

    /**
     * Returns a handle to the entity this trace hit
     *
     * @type {function}
     * @returns {CBaseEntity|null}
     */
    function GetEntity();

    /**
     * Returns the index of the entity hit, or -1 if it did not hit an entity
     *
     * @type {function}
     * @returns {int}
     */
    function GetEntityIndex();

    /**
     * Time completed, 1.0 means no hit
     *
     * @type {function}
     * @returns {float}
     */
    function GetFraction();

    /**
     * Returns the normal of the plane where the trace hit
     *
     * @type {function}
     * @returns {Vector}
     */
    function GetImpactNormal();

    /**
     * Returns the start position of the trace
     *
     * @type {function}
     * @returns {Vector}
     */
    function GetStartPos();

}

class CLight extends CBaseEntity
{
    /**
     * Returns the inner angle for spotlights.
     *
     * @type {function}
     * @returns {float}
     */
    function GetInnerAngle();

    /**
     * Returns the forward direction of the light for spotlights.
     *
     * @type {function}
     * @returns {Vector}
     */
    function GetLightForwardDirection();

    /**
     * Returns the outer angle for spotlights.
     *
     * @type {function}
     * @returns {float}
     */
    function GetOuterAngle();

    /**
     * Gets the radius override
     *
     * @type {function}
     * @returns {float}
     */
    function GetRadiusOverride();

    /**
     * Gets the shadow size.
     *
     * @type {function}
     * @returns {int}
     */
    function GetShadowSize();

    /**
     * Returns true if the light is enabled.
     *
     * @type {function}
     * @returns {bool}
     */
    function GetState();

    /**
     * Sets the cookie texture for the light.
     *
     * @type {function}
     * @param {string} cookie
     */
    function SetCookieTexture(cookie);

    /**
     * Sets the frame of the cookie texture for the light.
     *
     * @type {function}
     * @param {int} frame
     */
    function SetCookieTextureFrame(frame);

    /**
     * Sets the inner angle for spotlights.
     *
     * @type {function}
     * @param {float} angle
     */
    function SetInnerAngle(angle);

    /**
     * Set the light color.
     *
     * @type {function}
     * @param {Vector} color
     * @param {float} scale
     */
    function SetLightColor(color, scale);

    /**
     * Sets the Constant/Linear/Quadratic (CLQ) falloff ratios for the light.
     *
     * @type {function}
     * @param {float} constant
     * @param {float} linear
     * @param {float} quadratic
     */
    function SetLightFalloffCLQ(constant, linear, quadratic);

    /**
     * Sets the d50/d0 light fallfoff.
     *
     * @type {function}
     * @param {float} fiftyPercent
     * @param {float} zeroPercent
     */
    function SetLightFalloffD50D0(fiftyPercent, zeroPercent);

    /**
     * Sets the outer angle for spotlights.
     *
     * @type {function}
     * @param {float} angle
     */
    function SetOuterAngle(angle);

    /**
     * Sets the light pattern.
     *
     * @type {function}
     * @param {string} pattern
     */
    function SetPattern(pattern);

    /**
     * Sets the radius override, instead of computing it based on CQL or D0/D50
     *
     * @type {function}
     * @param {float} radius
     */
    function SetRadiusOverride(radius);

    /**
     * Sets the shadow size.
     *
     * @type {function}
     * @param {int} size
     */
    function SetShadowSize(size);

    /**
     * Sets the light's volumetric density
     *
     * @type {function}
     * @param {float} density
     */
    function SetVolumetricDensity(density);

    /**
     * Sets the light's contribution scale for volumetric lighting
     *
     * @type {function}
     * @param {float} scale
     */
    function SetVolumetricLightScale(scale);

    /**
     * Toggle the light.
     *
     * @type {function}
     */
    function Toggle();

    /**
     * Turn off the light.
     *
     * @type {function}
     */
    function TurnOff();

    /**
     * Turn on the light.
     *
     * @type {function}
     */
    function TurnOn();

}

class COBBVolumeFog extends CBaseEntity
{
    /**
     * Get the density of the fog
     *
     * @type {function}
     * @returns {float}
     */
    function GetDensity();

    /**
     * Get the emissive color of the fog
     *
     * @type {function}
     * @returns {Vector}
     */
    function GetEmissiveColor();

    /**
     * Get the half-size of the bounding box, before rotation
     *
     * @type {function}
     * @returns {Vector}
     */
    function GetHalfSize();

    /**
     * Get the phase of the fog
     *
     * @type {function}
     * @returns {float}
     */
    function GetPhase();

    /**
     * Get the scattering color of the fog
     *
     * @type {function}
     * @returns {Vector}
     */
    function GetScatteringColor();

    /**
     * Set the density of the fog
     *
     * @type {function}
     * @param {float} density
     */
    function SetDensity(density);

    /**
     * Set the emissive color of the fog
     *
     * @type {function}
     * @param {Vector} emissiveColor
     */
    function SetEmissiveColor(emissiveColor);

    /**
     * Set the half-size of the bounding box, before rotation
     *
     * @type {function}
     * @param {Vector} halfSize
     */
    function SetHalfSize(halfSize);

    /**
     * Set the phase of the fog
     *
     * @type {function}
     * @param {float} phase
     */
    function SetPhase(phase);

    /**
     * Set the scattering color of the fog
     *
     * @type {function}
     * @param {Vector} scatteringColor
     */
    function SetScatteringColor(scatteringColor);

    /**
     * Toggle the fog volume
     *
     * @type {function}
     */
    function Toggle();

    /**
     * Turn off the fog volume
     *
     * @type {function}
     */
    function TurnOff();

    /**
     * Turn on the fog volume
     *
     * @type {function}
     */
    function TurnOn();

}

class CEnvEntityMaker extends CBaseEntity
{
    /**
     * Create an entity at the location of the maker.
     *
     * @type {function}
     */
    function SpawnEntity();

    /**
     * Create an entity at the location of a specified entity instance.
     *
     * @type {function}
     * @param {CBaseEntity|null} target
     */
    function SpawnEntityAtEntityOrigin(target);

    /**
     * Create an entity at a specified location and orientaton, orientation is Euler angle in degrees (pitch, yaw, roll).
     *
     * @type {function}
     * @param {Vector} origin
     * @param {Vector} angles
     */
    function SpawnEntityAtLocation(origin, angles);

    /**
     * Create an entity at the location of a named entity.
     *
     * @type {function}
     * @param {string} name
     */
    function SpawnEntityAtNamedEntityOrigin(name);

}

/*
 * =======================
 * INSTANCES
 * =======================
 */


/**
 * Provides access to currently spawned entities.
 * @type {CEntities}
 * @const
*/
Entities <- CEntities()


/**
 * Contains the printed strings from the script_help command.
 * @type {table}
*/
Documentation <- {}