// This file is auto-generated by @hey-api/openapi-ts

import type { Options as ClientOptions, TDataShape, Client } from '@hey-api/client-fetch';
import type { GetAccountBalancesData, GetAccountBalancesResponse, GetAccountBalancesError, GetAccountIncomeStatementData, GetAccountIncomeStatementResponse, GetAccountIncomeStatementError, GetAccountNamesData, GetAccountNamesResponse, GetAccountNamesError, GetTransactionsData, GetTransactionsResponse, GetTransactionsError, PutAddData, PutAddResponse, PutAddError, GetCommoditiesData, GetCommoditiesResponse, GetCommoditiesError, GetVersionData, GetVersionResponse } from './types.gen';
import { client as _heyApiClient } from './client.gen';

export type Options<TData extends TDataShape = TDataShape, ThrowOnError extends boolean = boolean> = ClientOptions<TData, ThrowOnError> & {
    /**
     * You can provide a client instance returned by `createClient()` instead of
     * individual options. This might be also useful if you want to implement a
     * custom client.
     */
    client?: Client;
    /**
     * You can pass arbitrary values through the `meta` object. This can be
     * used to access values that aren't defined as part of the SDK function.
     */
    meta?: Record<string, unknown>;
};

export const getAccountBalances = <ThrowOnError extends boolean = false>(options: Options<GetAccountBalancesData, ThrowOnError>) => {
    return (options.client ?? _heyApiClient).get<GetAccountBalancesResponse, GetAccountBalancesError, ThrowOnError>({
        url: '/accountbalances/{filter}',
        ...options
    });
};

export const getAccountIncomeStatement = <ThrowOnError extends boolean = false>(options: Options<GetAccountIncomeStatementData, ThrowOnError>) => {
    return (options.client ?? _heyApiClient).post<GetAccountIncomeStatementResponse, GetAccountIncomeStatementError, ThrowOnError>({
        url: '/accountincomestatements/{filter}',
        ...options,
        headers: {
            'Content-Type': 'application/json',
            ...options?.headers
        }
    });
};

export const getAccountNames = <ThrowOnError extends boolean = false>(options?: Options<GetAccountNamesData, ThrowOnError>) => {
    return (options?.client ?? _heyApiClient).get<GetAccountNamesResponse, GetAccountNamesError, ThrowOnError>({
        url: '/accountnames',
        ...options
    });
};

export const getTransactions = <ThrowOnError extends boolean = false>(options: Options<GetTransactionsData, ThrowOnError>) => {
    return (options.client ?? _heyApiClient).get<GetTransactionsResponse, GetTransactionsError, ThrowOnError>({
        url: '/accounttransactions/{filter}',
        ...options
    });
};

export const putAdd = <ThrowOnError extends boolean = false>(options: Options<PutAddData, ThrowOnError>) => {
    return (options.client ?? _heyApiClient).put<PutAddResponse, PutAddError, ThrowOnError>({
        url: '/add',
        ...options,
        headers: {
            'Content-Type': 'application/json',
            ...options?.headers
        }
    });
};

export const getCommodities = <ThrowOnError extends boolean = false>(options?: Options<GetCommoditiesData, ThrowOnError>) => {
    return (options?.client ?? _heyApiClient).get<GetCommoditiesResponse, GetCommoditiesError, ThrowOnError>({
        url: '/commodities',
        ...options
    });
};

export const getVersion = <ThrowOnError extends boolean = false>(options?: Options<GetVersionData, ThrowOnError>) => {
    return (options?.client ?? _heyApiClient).get<GetVersionResponse, unknown, ThrowOnError>({
        url: '/version',
        ...options
    });
};