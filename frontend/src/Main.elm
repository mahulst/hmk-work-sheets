module Main exposing (Actions, DatabaseState(..), Model, Msg(..), Page(..), WorkDay, WorkSheet, actionsDecoder, filesDecoder, init, main, subscriptions, update, uploadView, view, viewAction, viewActions, viewDay, viewDuration, workDayDecoder, workSheetDecoder)

import Browser
import Browser.Navigation exposing (Key)
import Date exposing (Date)
import Dict exposing (Dict)
import File exposing (File)
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Http
import Json.Decode as D
import RemoteData exposing (RemoteData)
import Task
import Time
import Url exposing (Url)
import Url.Parser as Parser exposing ((</>), Parser, oneOf, s)



-- MAIN


main =
    Browser.application
        { init = init
        , onUrlChange = OnUrlChange
        , onUrlRequest = OnUrlRequest
        , view = view
        , update = update
        , subscriptions = subscriptions
        }



-- MODEL


type alias Actions =
    Dict String Int


type alias WorkDay =
    Dict String Actions


type alias WorkSheet =
    Dict String WorkDay


type alias Model =
    { page : Page
    , key : Key
    , currentDate : Date
    }


type Page
    = UploadingDatabase DatabaseState
    | Day DayModel
    | AvailableDays
    | PageNone


type Route
    = Upload
    | ViewDay String
    | SelectAvailableDay
    | NotFound


type DatabaseState
    = Waiting
    | Uploading Float
    | Done WorkSheet
    | Fail


type DayError
    = DayNotFound
    | ParseError


type alias DayModel =
    { day : String
    , workDay : RemoteData DayError WorkDay
    }


parseUrl : Url -> Route
parseUrl url =
    case Parser.parse routeParser url of
        Just route ->
            route

        Nothing ->
            NotFound


fetchDay : String -> Cmd Msg
fetchDay day =
    Http.get
        { url = "http://localhost:3010/day/" ++ day
        , expect = Http.expectJson GotDay workDayDecoder
        }


routeParser : Parser (Route -> a) a
routeParser =
    oneOf
        [ Parser.map ViewDay (s "dag" </> Parser.string)
        , Parser.map Upload (s "upload-registratie")
        , Parser.map SelectAvailableDay (s "kies-een-dag")
        ]


getPath : Route -> String
getPath route =
    case route of
        NotFound ->
            "/404"

        Upload ->
            "/upload-registratie"

        SelectAvailableDay ->
            "/kies-een-dag"

        ViewDay day ->
            "/dag/" ++ day


getPage : Route -> ( Page, Cmd Msg )
getPage route =
    case route of
        NotFound ->
            ( PageNone, Cmd.none )

        Upload ->
            ( UploadingDatabase Waiting, Cmd.none )

        SelectAvailableDay ->
            ( AvailableDays, Cmd.none )

        ViewDay day ->
            ( Day { day = day, workDay = RemoteData.NotAsked }, fetchDay day )


init : () -> Url -> Key -> ( Model, Cmd Msg )
init _ url key =
    let
        ( page, cmd ) =
            parseUrl url |> getPage

        currentDate =
            Date.fromCalendarDate 2019 Time.Jan 1
    in
    ( { page = page, key = key, currentDate = currentDate }
    , Cmd.batch [ Date.today |> Task.perform ReceiveDate, cmd ]
    )


workSheetDecoder : D.Decoder WorkSheet
workSheetDecoder =
    D.dict workDayDecoder


workDayDecoder : D.Decoder WorkDay
workDayDecoder =
    D.dict actionsDecoder


actionsDecoder : D.Decoder Actions
actionsDecoder =
    D.dict D.int



-- UPDATE


type Msg
    = GotFiles (List File)
    | GotProgress Http.Progress
    | Uploaded (Result Http.Error WorkSheet)
    | OnUrlRequest Browser.UrlRequest
    | OnUrlChange Url
    | ReceiveDate Date
    | GotDay (Result Http.Error WorkDay)
    | Noop


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        GotFiles files ->
            ( { model | page = UploadingDatabase (Uploading 0) }
            , Http.request
                { method = "POST"
                , url = "http://localhost:3010/upload"
                , headers = []
                , body = Http.multipartBody (List.map (Http.filePart "file") files)
                , expect = Http.expectJson Uploaded workSheetDecoder
                , timeout = Nothing
                , tracker = Just "upload"
                }
            )

        GotProgress progress ->
            case progress of
                Http.Sending p ->
                    ( { model | page = UploadingDatabase (Uploading (Http.fractionSent p)) }, Cmd.none )

                Http.Receiving _ ->
                    ( model, Cmd.none )

        Uploaded result ->
            case result of
                Ok response ->
                    ( { model | page = UploadingDatabase (Done response) }
                    , Browser.Navigation.pushUrl
                        model.key
                        (getPath SelectAvailableDay)
                    )

                Err _ ->
                    ( { model | page = UploadingDatabase Fail }, Cmd.none )

        OnUrlChange url ->
            let
                ( newPage, cmd ) =
                    parseUrl url |> getPage
            in
            ( { model | page = newPage }, cmd )

        Noop ->
            ( model, Cmd.none )

        ReceiveDate date ->
            ( { model | currentDate = date }, Cmd.none )

        OnUrlRequest urlRequest ->
            case urlRequest of
                Browser.Internal url ->
                    ( model
                    , Browser.Navigation.pushUrl model.key (Url.toString url)
                    )

                Browser.External url ->
                    ( model
                    , Browser.Navigation.load url
                    )

        GotDay (Ok workDay) ->
            let
                page =
                    Day
                        { workDay = RemoteData.Success workDay
                        , day = ""
                        }
            in
            ( { model | page = page }, Cmd.none )

        GotDay (Err err) ->
            let
                page =
                    Day
                        { workDay = RemoteData.Failure DayNotFound
                        , day = ""
                        }
            in
            ( { model | page = page }, Cmd.none )



-- SUBSCRIPTIONS


subscriptions : Model -> Sub Msg
subscriptions _ =
    Sub.batch
        [ Http.track "upload" GotProgress
        ]



-- VIEW


viewNavBar : Html Msg
viewNavBar =
    Html.nav [ class "navbar navbar-expand-lg navbar-dark bg-dark" ]
        [ Html.a [ class "navbar-brand" ] [ text "Humako uren registratie" ]
        , div [ class "collapse navbar-collapse show" ]
            [ Html.ul [ class "navbar-nav" ]
                [ Html.li
                    [ class "nav-item" ]
                    [ Html.a [ class "nav-link", href (getPath Upload) ] [ text "Upload nieuwe registratie" ] ]

                --                , Html.li
                --                    [ class "nav-item" ]
                --                    [ Html.a [ class "nav-link", href (getPath (ViewDay "2019-01-01")) ] [ text "Gister" ] ]
                --                , Html.li
                --                    [ class "nav-item" ]
                --                    [ Html.a [ class "nav-link", href (getPath (ViewDay "2019-01-02")) ] [ text "Vandaag" ] ]
                , Html.li
                    [ class "nav-item" ]
                    [ Html.a [ class "nav-link", href (getPath SelectAvailableDay) ] [ text "Selecteer dag" ] ]
                ]
            ]
        ]


view : Model -> Browser.Document Msg
view model =
    let
        header =
            [ viewNavBar ]
    in
    case model.page of
        UploadingDatabase database ->
            { title = "Upload uren registratie", body = header ++ [ uploadView database ] }

        Day day ->
            { title = "Uren voor " ++ day.day, body = header ++ [ viewDay day ] }

        AvailableDays ->
            { title = "Kies een dag"
            , body =
                header
                    ++ [ div []
                            [ datePickerView model.currentDate
                            ]
                       ]
            }

        PageNone ->
            { title = "404", body = [ div [] [ text "Pagina niet gevonden" ] ] }


getDateOffset : Date -> Int -> Date
getDateOffset date offset =
    Date.add Date.Days -offset date


datePickerView : Date -> Html Msg
datePickerView currentDate =
    let
        dayNumber =
            Date.weekdayNumber currentDate

        lastFourWeeks =
            List.range 0 (7 * 4 + dayNumber - 1)

        getDateOffsetToCurrent =
            getDateOffset currentDate

        dates =
            List.map getDateOffsetToCurrent lastFourWeeks |> List.reverse
    in
    div [ class "datepicker--days" ]
        (List.map
            (\date ->
                a
                    [ class "datepicker--day"
                    , href
                        (getPath
                            (ViewDay
                                (Date.format "y-MM-dd"
                                    date
                                )
                            )
                        )
                    ]
                    [ text (Date.format "d" date) ]
            )
            dates
        )


uploadView : DatabaseState -> Html Msg
uploadView database =
    case database of
        Waiting ->
            input
                [ type_ "file"
                , multiple True
                , on "change" (D.map GotFiles filesDecoder)
                ]
                []

        Uploading fraction ->
            h1 [] [ text (String.fromInt (round (100 * fraction)) ++ "%") ]

        Done _ ->
            Html.div [] [ h1 [] [ text "DONE" ] ]

        Fail ->
            h1 [] [ text "FAILED IMPORTING DATABASE" ]


viewDuration : Int -> String
viewDuration duration =
    let
        minutes =
            remainderBy 60 duration |> String.fromInt

        formattedMinutes =
            if String.length minutes == 1 then
                "0" ++ minutes

            else
                minutes

        hours =
            floor (toFloat duration / 60) |> String.fromInt
    in
    hours ++ ":" ++ formattedMinutes


viewAction : String -> ( String, Int ) -> Html msg
viewAction employee ( action, duration ) =
    Html.tr []
        [ Html.td [] [ text employee ]
        , Html.td [] [ text action ]
        , Html.td [] [ text (viewDuration duration) ]
        ]


viewActions : ( String, Actions ) -> List (Html msg)
viewActions ( employee, actions ) =
    let
        rows =
            Dict.toList actions
                |> List.map (viewAction employee)

        total =
            Dict.toList actions |> List.map Tuple.second |> List.sum

        totalRow =
            Html.tr [class "table-secondary"]
                [ Html.td [] [ text employee ]
                , Html.td [] [ text "Totaal" ]
                , Html.td [] [ text (viewDuration total) ]
                ]
    in
    rows ++ [totalRow]


viewDay : DayModel -> Html msg
viewDay dayModel =
    case dayModel.workDay of
        RemoteData.NotAsked ->
            div [] [ text "Laden..." ]

        RemoteData.Loading ->
            div [] [ text "Laden..." ]

        RemoteData.Success workDay ->
            Html.table [ class "workday--table table table-sm table-hover" ]
                [ Html.thead []
                    [ Html.tr []
                        [ Html.th [ scope "col" ] [ text "Werknemer" ]
                        , Html.th [ scope "col" ] [ text "Taak" ]
                        , Html.th [ scope "col" ] [ text "Tijd" ]
                        ]
                    ]
                , Html.tbody []
                    (Dict.toList workDay
                        |> List.map viewActions
                        |> List.concat
                    )
                ]

        RemoteData.Failure _ ->
            div [] [ text "Voor deze dag is geen uren registratie beschikbaar" ]


filesDecoder : D.Decoder (List File)
filesDecoder =
    D.at [ "target", "files" ] (D.list File.decoder)
