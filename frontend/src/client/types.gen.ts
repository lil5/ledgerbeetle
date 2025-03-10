// This file is auto-generated by @hey-api/openapi-ts

export type AddTransaction = {
    /**
     * amount added to debit account
     */
    amount: number;
    /**
     * transaction code
     */
    code: number;
    /**
     * commodity used
     */
    commodityUnit: string;
    /**
     * account name
     */
    creditAccount: string;
    /**
     * account name
     */
    debitAccount: string;
    /**
     * random hex u128 string
     */
    relatedId: string;
};

export type AddTransactions = {
    /**
     * unix time milliseconds
     */
    fullDate2: number;
    /**
     * list of transactions
     */
    transactions: Array<AddTransaction>;
};

export type Balance = {
    accountName: string;
    amount: number;
    commodityDecimal: number;
    commodityUnit: string;
};

export type GetAccountIncomeStatementBody = {
    dates: Array<number>;
};

export type IncomeStatement = {
    accountName: string;
    amounts: Array<number>;
    commodityDecimal: number;
    commodityUnit: string;
};

export type ResponseIncomeStatements = {
    dates: Array<number>;
    incomeStatements: Array<IncomeStatement>;
};

export type Transaction = {
    /**
     * transaction code
     */
    code: number;
    /**
     * location of decimal point
     */
    commodityDecimal: number;
    /**
     * commodity used
     */
    commodityUnit: string;
    /**
     * account name
     */
    creditAccount: string;
    /**
     * amount removed from credit account
     */
    creditAmount: number;
    /**
     * account name
     */
    debitAccount: string;
    /**
     * amount added to debit account
     */
    debitAmount: number;
    /**
     * unix time milliseconds
     */
    fullDate: number;
    /**
     * unit time milliseconds
     */
    fullDate2: number;
    /**
     * random hex u128 string
     */
    relatedId: string;
    /**
     * random hex u128 string
     */
    transferId: string;
};

export type Vec = Array<string>;

export type GetAccountBalancesData = {
    body?: never;
    path: {
        filter: string;
    };
    query?: {
        date?: number | null;
    };
    url: '/accountbalances/{filter}';
};

export type GetAccountBalancesErrors = {
    /**
     * Bad request error occurred
     */
    400: string;
    /**
     * Internal server error occurred
     */
    500: string;
};

export type GetAccountBalancesError = GetAccountBalancesErrors[keyof GetAccountBalancesErrors];

export type GetAccountBalancesResponses = {
    /**
     * Returns list of account balances by filter
     */
    200: Array<Balance>;
};

export type GetAccountBalancesResponse = GetAccountBalancesResponses[keyof GetAccountBalancesResponses];

export type GetAccountIncomeStatementData = {
    body: GetAccountIncomeStatementBody;
    path: {
        filter: string;
    };
    query?: never;
    url: '/accountincomestatements/{filter}';
};

export type GetAccountIncomeStatementErrors = {
    /**
     * Bad request error occurred
     */
    400: string;
    /**
     * Internal server error occurred
     */
    500: string;
};

export type GetAccountIncomeStatementError = GetAccountIncomeStatementErrors[keyof GetAccountIncomeStatementErrors];

export type GetAccountIncomeStatementResponses = {
    /**
     * Returns list of balances by filter by date
     */
    200: ResponseIncomeStatements;
};

export type GetAccountIncomeStatementResponse = GetAccountIncomeStatementResponses[keyof GetAccountIncomeStatementResponses];

export type GetAccountNamesData = {
    body?: never;
    path?: never;
    query?: never;
    url: '/accountnames';
};

export type GetAccountNamesErrors = {
    /**
     * Bad request error occurred
     */
    400: string;
    /**
     * Internal server error occurred
     */
    500: string;
};

export type GetAccountNamesError = GetAccountNamesErrors[keyof GetAccountNamesErrors];

export type GetAccountNamesResponses = {
    /**
     * Returns list of transaction ids
     */
    200: Array<string>;
};

export type GetAccountNamesResponse = GetAccountNamesResponses[keyof GetAccountNamesResponses];

export type GetTransactionsData = {
    body?: never;
    path: {
        filter: string;
    };
    query: {
        date_newest: number;
        date_oldest: number;
    };
    url: '/accounttransactions/{filter}';
};

export type GetTransactionsErrors = {
    /**
     * Bad request error occurred
     */
    400: string;
    /**
     * Internal server error occurred
     */
    500: string;
};

export type GetTransactionsError = GetTransactionsErrors[keyof GetTransactionsErrors];

export type GetTransactionsResponses = {
    /**
     * Returns list of transactions by filter
     */
    200: Array<Transaction>;
};

export type GetTransactionsResponse = GetTransactionsResponses[keyof GetTransactionsResponses];

export type PutAddData = {
    body: AddTransactions;
    path?: never;
    query?: never;
    url: '/add';
};

export type PutAddErrors = {
    /**
     * Bad request error occurred
     */
    400: string;
    /**
     * Internal server error occurred
     */
    500: string;
};

export type PutAddError = PutAddErrors[keyof PutAddErrors];

export type PutAddResponses = {
    /**
     * Returns list of transaction ids
     */
    200: Vec;
};

export type PutAddResponse = PutAddResponses[keyof PutAddResponses];

export type GetCommoditiesData = {
    body?: never;
    path?: never;
    query?: never;
    url: '/commodities';
};

export type GetCommoditiesErrors = {
    /**
     * Bad request error occurred
     */
    400: string;
    /**
     * Internal server error occurred
     */
    500: string;
};

export type GetCommoditiesError = GetCommoditiesErrors[keyof GetCommoditiesErrors];

export type GetCommoditiesResponses = {
    /**
     * Returns list of commodities
     */
    200: Array<string>;
};

export type GetCommoditiesResponse = GetCommoditiesResponses[keyof GetCommoditiesResponses];

export type GetVersionData = {
    body?: never;
    path?: never;
    query?: never;
    url: '/version';
};

export type GetVersionResponses = {
    /**
     * Returns crate version
     */
    200: string;
};

export type GetVersionResponse = GetVersionResponses[keyof GetVersionResponses];

export type ClientOptions = {
    baseUrl: 'http://localhost:5173' | (string & {});
};