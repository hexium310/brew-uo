const marked = require('marked');
const { diff } = require('deep-object-diff');

const buildBody = (content) => {
  return [
    '```json',
    content,
    '```'
  ].join('\n');
};

const createIssue = async (github, context, content) => {
  const { owner, repo } = context.repo;

  const { data: issue } = await github.rest.issues.create({
    owner,
    repo,
    title: 'Unsupported version delimiters',
    body: buildBody(content),
    label: 'unsupported-version',
  });

  console.log(`Create ${ issue.html_url }`);
};

const comment = async (github, context, issue, content) => {
  const { owner, repo } = context.repo;

  const issueNumber = issue.number;
  const { data: comments } = await github.rest.issues.listComments({
    owner,
    repo,
    issue_number: issueNumber,
  });
  const bodies = [issue.body, comments.map(({ body }) => body)].flat();

  const mentionedVersions = Object.assign({}, ...bodies.flatMap((body) => {
    const tokens = marked.lexer(body);
    const codes = tokens.filter(({ type, lang }) => type === 'code' && lang === 'json');
    return codes.map(({ text }) => JSON.parse(text));
  }));
  const versions = JSON.parse(content);

  const diffKeys = Object.keys(diff(mentionedVersions, versions));
  if (diffKeys.length === 0) {
    return;
  }

  const diffVersions = Object.fromEntries(Object.entries(versions).filter(([key, _]) => diffKeys.includes(key)));
  const { data: comment } = await github.rest.issues.createComment({
    owner,
    repo,
    issue_number: issueNumber,
    body: buildBody(JSON.stringify(diffVersions, null, 2)),
  });

  console.log(`Comment ${ comment.html_url }`);
};

module.exports = async ({ github, context, core }) => {
  const { VERSIONS } = process.env;
  const { owner, repo } = context.repo;

  if (Object.keys(JSON.parse(VERSIONS)).length === 0) {
    return;
  }

  const { data: issues } = await github.rest.issues.listForRepo({
    owner,
    repo,
    state: 'open',
    label: 'unsupported-version',
  });

  if (issues.length === 0) {
    await createIssue(github, context, VERSIONS);
    return;
  }

  const latestIssue = issues[issues.length - 1];
  await comment(github, context, latestIssue, VERSIONS);
}
