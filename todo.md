# Development Notes

## Project Design

- Slack bot listens for `DELIVER` and `REQUEST_WORK` events.
- On event, HTTP Server is sent a JSON info packet.

- HTTP Server (Axum) 
    - DELIVER
        - body: { user }
        - Retrieve movies
- Database server
    - Postgres

- Roles: [ AE, Editor, Sound, Finish ]

## Slack requirements
I think it'd be nice for students to see - instead of using slash commands, let's
have shereebot listen in a project channel OR DM's.

- Student can call `/deliver` to complete work.
    - Get `student.id, students_films.film_id, film.id,`
    - Get `student_roles.current, film_roles.current`

    - Update `student.roles.current, student.roles.[role]`
    - Update `film.roles.current, film.roles.[role]`

    - Add `{student.slack_id, film_name, roles.current.next_role()}` to `jobs` queue.

    - "Congrats! You've finished work for ROLE. When you're ready, use /request-work to get more."

- `/request-work`
    - Get `student.id, students_films.film_id, film.id,`
        - we need to retrieve student, all films they've worked, all roles they've worked.
    - Get `student_roles.current, film_roles.current`

    - Query `jobs` queue for next available film for `roles.current`
        - If none, add `{student.slack_id, film_name, role}` to the `wait` queue.
            - error if student in queue already
        - If yes, dequeue film from `jobs` queue.
            - Insert to join table `student_films` 
            - 



## Student facing reqs:

Requirements:
    - Priority queue of movies (HIGH LOW priority (only two priorities))
    - DELIVER button
        - When student finishes work, adds the movie to the available work queue.
        - Each time DELIVER is clicked, also check the waiting queue for waiting students
        - If there are waiting students, give them a movie from the work queue
    - REQUEST_WORK button
        - Polls from the movie queue and assigns movie to student.
        - If no movies are available, put student in WAIT queue - they will be assigned when next DELIVER 


Teacher facing side:
    - Show status of every movie

## Data layer
- [Diagram](https://sqldbm.com/Project/Dashboard/All/)

students
    - id
    - email
    - FK films_worked_on
    - FK prod_roles_worked

films
    - id
    - name
    - FK prod_roles

roles
    - ae
    - editor
    - sound
    - finish

student_films
    - FK student_id
    - FK film_id

student_roles
    - FK student_id
    - FK role_id

TODO:
    - Pass slack id in as channel to directly PM 
    - Let sheree @ slackbot with a csv file
    - Then we can read the private URL to grab its data.
