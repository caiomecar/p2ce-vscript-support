const _charsize_ = 1;

const _floatsize_ = 4;

const _intsize_ = 8;

const _version_ = "Squirrel 3.2 stable";

const _versionnumber_ = 320;

/**
 * @param {bool} exp
 * @throws {string}
 */
function assert(exp);

function castf2i(value);

function casti2f(value);

function collectgarbage();

function compilestring(code, buffer_name = null);

function dummy(...);

function enabledebuginfo(enable);

function error(message);

function getconsttable();

function getroottable();

function getstackinfos();

function print(message);

function resurrectunreachable();

function setconsttable(const_table);

function setdebughook(hook_func);

function seterrorhandler(error_func);

function setroottable(table);

function swap2(value);

function swap4(value);

function swapfloat(value);

function type(value);



// Math

function abs(value);

function acos(value);

function asin(value);

function atan(value);

function atan2(y, x);

function ceil(value);

function cos(value);

function exp(value);

function fabs(value);

function floor(value);

function log(value);

function log10(value);

function pow(value, exponent);

function rand();

function sin(value);

function sqrt(value);

function srand(seed);

function tan(value);



// Strings

function endswith(str, cmp);

function escape(str, cmp);

function format(str, ...);

function lstrip(str);

function rstrip(str);

function split(str, separator, skip_empty = false);

function startswith(str, cmp);

function strip(str);



// Extra

/**
 * @param {int} length
 * @param {any} fill
 * @returns {array}
 */
function array(length, fill = null);

/**
 * @param {function} func
 * @returns {thread}
 */
function newthread(func);

function suspend(return_value = this);



// Classes

class regexp {
    /**
     * @param {string} pattern
     */
    constructor(pattern);

    /**
     * @param {string} str
     * @param {integer} start
     * @returns {table}
     */
    function capture(str, start = 0);

    /**
     * @param {string} str
     * @return {bool}
     */
    function match(str);

    /**
     * @param {string} str
     * @param {integer} start
     * @returns {table}
     */
    function search(str, start = 0);

    /**
     * @returns {integer}
     */
    function subexpcount();
}

class blob {
    function eos();

    function flush();

    function len();

    function readblob(num_of_bytes);

    function readn(data_type);

    function resize(new_size);

    function seek(offset, offset_basis);

    function swap2();

    function swap4();

    function tell();

    function writeblob(blob);

    function writen(value, data_type);

    function writestring(str);
}