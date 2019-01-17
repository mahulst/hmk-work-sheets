module Main exposing (..)

import Browser
import Dict exposing (Dict)
import File exposing (File)
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Http
import Json.Decode as D


-- MAIN


main =
    Browser.element
        { init = init
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


type Model
    = Waiting
    | Uploading Float
    | Done WorkSheet
    | Fail


init : () -> ( Model, Cmd Msg )
init _ =
    ( Waiting
    , Cmd.none
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


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        GotFiles files ->
            ( Uploading 0
            , Http.request
                { method = "POST"
                , url = "/upload/"
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
                    ( Uploading (Http.fractionSent p), Cmd.none )

                Http.Receiving _ ->
                    ( model, Cmd.none )

        Uploaded result ->
            case result of
                Ok response ->
                    ( Done response, Cmd.none )

                Err _ ->
                    ( Fail, Cmd.none )



-- SUBSCRIPTIONS


subscriptions : Model -> Sub Msg
subscriptions model =
    Http.track "upload" GotProgress



-- VIEW


view : Model -> Html Msg
view model =
    case model of
        Waiting ->
            input
                [ type_ "file"
                , multiple True
                , on "change" (D.map GotFiles filesDecoder)
                ]
                []

        Uploading fraction ->
            h1 [] [ text (String.fromInt (round (100 * fraction)) ++ "%") ]

        Done worksheet ->
            Html.div [] [ h1 [] [ text "DONE" ], viewWorkSheet worksheet ]

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


viewAction : ( String, Int ) -> Html msg
viewAction ( action, duration ) =
    Html.tr [ class "action-row" ] [ Html.td [ class "action-title" ] [ text action ], Html.td [ class "duration" ] [ text (viewDuration duration) ] ]


viewActions : ( String, Actions ) -> Html msg
viewActions ( employee, actions ) =
    Html.tr []
        [ Html.td [ class "employee-name" ] [ text employee ]
        , Html.td []
            [ Html.table [ class "action" ]
                (Dict.toList actions
                    |> List.map viewAction
                )
            ]
        ]


viewDay : ( String, WorkDay ) -> Html msg
viewDay ( day, workDay ) =
    Html.tr [ class "day-row"]
        [ Html.td [ class "day-title" ] [ text day ]
        , Html.td []
            [ Html.table [ class "employee" ]
                (Dict.toList workDay
                    |> List.map viewActions
                )
            ]
        ]


viewWorkSheet : WorkSheet -> Html msg
viewWorkSheet worksheet =
    Html.table [ class "work-sheet" ]
        (Dict.toList worksheet
            |> List.map viewDay
        )


filesDecoder : D.Decoder (List File)
filesDecoder =
    D.at [ "target", "files" ] (D.list File.decoder)
