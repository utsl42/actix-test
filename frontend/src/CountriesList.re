type name = {common: string};
type listCountries = {
  name,
  cca3: string,
};

/* Create a GraphQL Query by using the graphql_ppx */
module GetCountriesList = [%graphql
  {|
  query get_countries {
    listCountries @bsRecord {
      name @bsRecord {
        common
      }
      cca3
    }
  }
|}
];

module GetCountriesQuery = ReasonApollo.CreateQuery(GetCountriesList);

let str = ReasonReact.string;
let component = ReasonReact.statelessComponent("CountriesList");

let push = (path, event) => {
  ReactEvent.Mouse.preventDefault(event);
  ReasonReact.Router.push("#" ++ path);
};

let make = (~items: array(listCountries), _children) => {
  ...component,
  render: _self =>
    <ul style={ReactDOMRe.Style.make(~listStyleType="none", ())}>
      {
        items
        |> Array.map(item => <li key={item.cca3} onClick={push(item.cca3)}> {item.name.common |> str} </li>)
        |> ReasonReact.array
      }
    </ul>,
};
