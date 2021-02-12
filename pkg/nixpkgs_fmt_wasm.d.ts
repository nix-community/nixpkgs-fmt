/* tslint:disable */
/* eslint-disable */
/**
*/
export function main(): void;
/**
* @param {string} text 
* @param {string} format 
* @returns {string} 
*/
export function reformat_string(text: string, format: string): string;

/**
* If `module_or_path` is {RequestInfo}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {RequestInfo | BufferSource | WebAssembly.Module} module_or_path
*
* @returns {Promise<any>}
*/
export default function init (module_or_path?: RequestInfo | BufferSource | WebAssembly.Module): Promise<any>;
        