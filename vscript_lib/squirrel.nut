/**
 * Squirrel Standard Library Signatures
 * Generated from https://wiki.teamfortress.com/wiki/Team_Fortress_2/Scripting/Script_Functions
 * Only for reference, do not modify
 * @native
 */

/**
 * 32-bit: `4`
 *
 * 64-bit: `8`
 */
const _intsize_ = 8;
const _charsize_ = 1;
const _floatsize_ = 4;


const _version_ = "Squirrel 3.2 stable";
const _versionnumber_ = 320;

/**
 * Windows: `32767`
 *
 * Linux: `2147483647`
 */
const RAND_MAX = 2147483647

const PI = 3.14159

/**
 * Throws an assertion error if the given expression evaluates to false (i.e. the values `0`, `0.0`, `-0.0`, `null` and `false`).
 * @type {function}
 * @param {any} exp
 * @throws {string} if the given expression evaluates to false
 */
function assert(exp);

/**
 * Interprets the float's bytes as if it were a 32-bit integer representation.
 * @type {function}
 * @param {float} value
 * @returns {integer}
 */
function castf2i(value);

/**
 * Interprets the integer's bytes as if it were a floating-point encoding.
 * @type {function}
 * @param {integer} value
 * @returns {float}
 */
function casti2f(value);

/**
 * Runs the garbage collector and returns the number of reference cycles found and deleted.
 * @type {function}
 * @returns {integer}
 */
function collectgarbage();

/**
 * Compiles a string containing a squirrel script into a function and returns it.
 * @type {function}
 * @param {string} code
 * @param {string|null} buffer_name Defaults to `null`
 * @returns {function}
 */
function compilestring(code, buffer_name = null);

/**
 * Returns and does nothing. Can be used as an empty function placeholder.
 * @type {function}
 * @varargs {any}
 */
function dummy(...);

/**
 * Enable or disable debug line information generation at compile time.
 * @type {function}
 * @param {bool} enable
 */
function enabledebuginfo(enable);

/**
 * Prints message to the standard error output.
 * @type {function}
 * @param {any} message
 */
function error(message);

/**
 * Returns the const table of the VM.
 * @type {function}
 * @returns {table}
 */
function getconsttable();

/**
 * Returns the root table of the VM.
 * @type {function}
 * @returns {table}
 */
function getroottable();

/**
 * Returns the stack frame information at the given stack level.
 * `0` is the current function, `1` is the caller and so on.
 * @type {function}
 * @param {integer} level
 * @returns {table|null} `null` if the stack level doesn't exist.
 */
function getstackinfos(level);

/**
 * Prints the given parameter with no newline.
 * @type {function}
 * @param {any} message
 */
function print(message);

/**
 * Runs the garbage collector and returns an array containing all unreachable objects found.
 * @type {function}
 * @returns {[any]} `null` if none are found.
 */
function resurrectunreachable();

/**
 * Sets the const table of the VM and returns the previous const table.
 * @type {function}
 * @param {table} const_table
 * @returns {table}
 */
function setconsttable(const_table);

/**
 * Sets the debug hook.
 * @type {function}
 * @param {function} hook_func
 */
function setdebughook(hook_func);

/**
 * Sets the runtime error handler.
 * @type {function}
 * @param {function|null} error_func `@(error: any)`
 */
function seterrorhandler(error_func);

/**
 * Sets the root table of the VM and returns the previous root table.
 * @type {function}
 * @param {table} table
 * @returns {table}
 */
function setroottable(table);

/**
 * Swaps bytes 1 and 2 of the integer.
 * @type {function}
 * @param {integer} value
 * @returns {integer}
 */
function swap2(value);

/**
 * Reverses the byte order of the four bytes of an integer.
 * @type {function}
 * @param {integer} value
 * @returns {integer}
 */
function swap4(value);

/**
 * Reverses the byte order of the four bytes of a float.
 * @type {function}
 * @param {float} value
 * @returns {float}
 */
function swapfloat(value);

/**
 * Returns the native type of the given parameter as a string, bypassing the `_typeof` delegate method.
 * @type {function}
 * @param {any} value
 * @returns {string}
 */
function type(value);

/**
 * Returns `|x|` as integer.
 * @type {function}
 * @param {float} x
 * @returns {integer}
 */
function abs(x);

/**
 * Returns the arc cosine of `x` in radians.
 * @type {function}
 * @param {float} x
 * @returns {float}
 */
function acos(x);

/**
 * Returns the arc sine of `x` in radians.
 * @type {function}
 * @param {float} x
 * @returns {float}
 */
function asin(x);

/**
 * Returns the arc tangent of `x` in radians.
 * @type {function}
 * @param {float} x
 * @returns {float}
 */
function atan(x);

/**
 * Returns the angle between the ray from `(0,0)` through `(x,y)` and the positive x-axis, in the range `(-PI, PI]`.
 * @type {function}
 * @param {float} y
 * @param {float} x
 * @returns {float}
 */
function atan2(y, x);

/**
 * Returns the smallest integer that is `>= x` as a float.
 * @type {function}
 * @param {float} x
 * @returns {float}
 */
function ceil(x);

/**
 * Returns the cosine of `x`.
 * @type {function}
 * @param {float} x
 * @returns {float}
 */
function cos(x);

/**
 * Returns `e` raised to the power of `x`.
 * @type {function}
 * @param {float} x
 * @returns {float}
 */
function exp(x);

/**
 * Returns `|x|` as float.
 * @type {function}
 * @param {float} x
 * @returns {float}
 */
function fabs(x);

/**
 * Returns the largest integer that is `<= x` as a float.
 * @type {function}
 * @param {float} x
 * @returns {float}
 */
function floor(x);

/**
 * Returns the natural logarithm of `x`.
 * @type {function}
 * @param {float} x
 * @returns {float}
 */
function log(x);

/**
 * Returns the base-10 logarithm of `x`.
 * @type {function}
 * @param {float} x
 * @returns {float}
 */
function log10(x);

/**
 * Returns `x` raised to the power of `y`.
 * @type {function}
 * @param {float} x
 * @param {float} y
 * @returns {float}
 */
function pow(x, y);

/**
 * Returns a random integer where `0 <= rand() <= RAND_MAX`.
 * @type {function}
 * @returns {integer}
 */
function rand();

/**
 * Returns the sine of `x`.
 * @type {function}
 * @param {float} x
 * @returns {float}
 */
function sin(x);

/**
 * Returns the square root of `x`.
 * @type {function}
 * @param {float} x
 * @returns {float}
 */
function sqrt(x);

/**
 * Sets the starting point for generating a series of pseudorandom integers.
 * @type {function}
 * @param {integer} seed
 */
function srand(seed);

/**
 * Returns the tangent of x.
 * @type {function}
 * @param {float} x
 * @returns {float}
 */
function tan(x);

/**
 * Returns `true` if `str` ends with `cmp`.
 * @type {function}
 * @param {string} str
 * @param {string} cmp
 * @returns {bool}
 */
function endswith(str, cmp);

/**
 * Returns a string with backslashes inserted before characters that need to be escaped.
 * @type {function}
 * @param {string} str
 * @returns {string}
 */
function escape(str);

/**
 * Returns a formatted string using printf-style format specifiers.
 * @type {function}
 * @param {string} str
 * @varargs {any}
 * @returns {string}
 */
function format(str, ...);

/**
 * Removes whitespace from the beginning of the string.
 * @type {function}
 * @param {string} str
 * @returns {string}
 */
function lstrip(str);

/**
 * Removes whitespace from the end of the string.
 * @type {function}
 * @param {string} str
 * @returns {string}
 */
function rstrip(str);

/**
 * Returns an array of strings split at each occurrence of a separator character.
 * @type {function}
 * @param {string} str
 * @param {string} separator
 * @param {bool} skip_empty Defaults to `skip_empty`
 * @returns {[string]}
 */
function split(str, separator, skip_empty = false);

/**
 * Returns `true` if `str` starts with `cmp`.
 * @type {function}
 * @param {string} str
 * @param {string} cmp
 * @returns {bool}
 */
function startswith(str, cmp);

/**
 * Removes whitespace from both the beginning and end of the string.
 * @type {function}
 * @param {string} str
 * @returns {string}
 */
function strip(str);

/**
 * Returns a new array of the given length where each element is set to `fill`.
 * @type {function}
 * @param {integer} length
 * @param {any} fill Defaults to `null`
 * @returns {[any]}
 */
function array(length, fill = null);

/**
 * Creates a new cooperative thread object and returns it.
 * @type {function}
 * @param {function} func
 * @returns {thread}
 */
function newthread(func);

/**
 * Suspends the coroutine that called this function.
 * @type {function}
 * @param {any} return_value Defaults to `null` which will be converted to `this`
 * @returns {any}
 */
function suspend(return_value = null);

/** @type {class} */
class regexp {
    /**
     * Creates and compiles a regular expression from the given pattern.
     * @type {function}
     * @param {string} pattern
     */
    constructor(pattern);

    /**
     * Returns an array of tables with `"begin"` and `"end"` keys for the first match and each captured sub-expression.
     * @type {function}
     * @param {string} str
     * @param {integer} start Defaults to `0`
     * @returns {table|null} `null` if no match occurs.
     */
    function capture(str, start = 0);

    /**
     * Returns `true` if the regular expression matches the entire string.
     * @type {function}
     * @param {string} str
     * @returns {bool}
     */
    function match(str);

    /**
     * Returns a table with `"begin"` and `"end"` keys for the first match in str, or `null` if no match occurs.
     * @type {function}
     * @param {string} str
     * @param {integer} start
     * @returns {table|null}
     */
    function search(str, start = 0);

    /**
     * Returns the number of sub-expression groups in the regular expression. Always `>= 1` since the whole regex counts as a group.
     * @type {function}
     * @returns {integer}
     */
    function subexpcount();
}

/** @type {class} */
class blob {
    /**
     * Creates a new blob of the given initial size.
     * @type {function}
     * @param {integer} init_size Defaults to `0`
     */
    constructor(init_size = 0);

    /**
     * Returns non-zero if the current read/write position is at the end of the blob.
     * @type {function}
     * @returns {integer}
     */
    function eos();

    /**
     * Flushes the blob stream.
     * @type {function}
     */
    function flush();

    /**
     * Returns the size of the blob in bytes.
     * @type {function}
     * @returns {integer}
     */
    function len();

    /**
     * Reads the specified number of bytes and returns them as a new blob.
     * @type {function}
     * @param {integer} num_of_bytes
     * @returns {blob}
     */
    function readblob(num_of_bytes);

    /**
     * Reads a number from the blob according to the data type character.
     * @type {function}
     * @param {integer} data_type
     * @returns {any}
     */
    function readn(data_type);

    /**
     * Resizes the blob to the specified size.
     * @type {function}
     * @param {integer} new_size
     */
    function resize(new_size);

    /**
     * Moves the read/write position. Returns
     * @type {function}
     * @param {integer} offset
     * @param {integer} offset_basis
     * @returns {integer} `0` on success.
     */
    function seek(offset, offset_basis);

    /**
     * Swaps the byte order of all 2-byte aligned values in the blob.
     * @type {function}
     */
    function swap2();

    /**
     * Swaps the byte order of all 4-byte aligned values in the blob.
     * @type {function}
     */
    function swap4();

    /**
     * Returns the current read/write position.
     * @type {function}
     * @returns {integer}
     */
    function tell();

    /**
     * Writes the contents of another blob into this blob at the current position.
     * @type {function}
     * @param {blob} src
     */
    function writeblob(src);

    /**
     * Writes a number to the blob according to the data type character.
     * @type {function}
     * @param {any} value
     * @param {integer} data_type
     */
    function writen(value, data_type);

    /**
     * Writes a string into the blob at the current position.
     * @type {function}
     * @param {string} str
     */
    function writestring(str);
}