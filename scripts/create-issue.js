const marked = require('marked');
const { diff } = require('deep-object-diff');

const buildBody = (content) => {
  return [
    '```json',
    content,
    '```'
  ].join('\n');
};

const createIssue = async (content) => {
  const issue = await github.rest.issues.create({
    owner,
    repo,
    title: 'Unsupported version delimiters',
    body: buildBody(content),
    label: 'unsupported-version',
  });

  console.log(`Create ${ issue.html_url }`);
};

const comment = async (issue, content) => {
  const issueNumber = issue.number;
  const comments = await github.rest.issues.listComments({
    owner,
    repo,
    issue_number: issueNumber,
  });
  const bodies = [issue.body, comments.map(({ body }) => body)];
  const mentionedVersions = bodies.map((body) => {
    const tokens = marked.lexer(body);
    const codes = tokens.filter(({ type, lang }) => type === 'code' && lang === 'json');
    return codes.map(({ text }) => JSON.parse(text));
  }).flat();
  const versions = JSON.parse(content);
  const diffKeys = Object.keys(diff(mentionedVersions, versions));
  const diffVersions = versions.filter((obj) => diffKeys.includes(Object.keys(obj)[0]));
  const comment = await github.rest.issues.createComment({
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

  const issues = await github.rest.issues.listForRepo({
    owner,
    repo,
    state: 'open',
    label: 'unsupported-version',
  });

  if (issues.length === 0) {
    await createIssue(VERSIONS);
    return;
  }


  const latestIssue = issues[issues.lenght - 1];
  await comment(latestIssue, VERSIONS);
}
