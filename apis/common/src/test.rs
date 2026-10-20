// schema:
//     title: Retrieve a list of all business details for Subcontractor
//     description: Retrieve business details response
//     required:
//     - listOfBusinesses
//     type: object
//     properties:
//         listOfBusinesses:
//         description: An array containing the details of all the business income sources held against the specified National insurance number.
//         type: array
//         items:
//             required:
//             - businessId
//             - typeOfBusiness
//             type: object
//             properties:
//             businessId:
//                 description: A unique identifier for the business income source. Previously known as selfEmploymentId.
//                 pattern: ^X[a-zA-Z0-9]{1}IS[0-9]{11}$
//                 type: string
//                 example: XAIS12345678910
//             tradingName:
//                 description: The trading name of the business.
//                 maxLength: 105
//                 type: string
//                 example: RCDTS
//             tradingType:
//                 description: The type of trading carried out by the business.
//                 maxLength: 35
//                 minLength: 1
//                 type: string
//                 example: Plastering
//             typeOfBusiness:
//                 description: The type of business income source.
//                 enum:
//                 - self-employment
//                 - uk-property
//                 - foreign-property
//                 - property-unspecified
//                 type: string
//                 example: self-employment
//             additionalProperties: false
//     additionalProperties: false

// 1. Types don't have names
//    A. Autogenerate a name
//    B. Let the user provide a name (override)

// 2. Convert property naming conventions

// 3. Descriptions should be copied into documentation.

// 4. Specialisation of types (generate the validation code.)
//    A. Allow the user to opt out.
//

// 5. Unify Common Types

// 200 OK

