/**
 * @param {bool} exp
 * @throws {string}
 */
function assert(exp);

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